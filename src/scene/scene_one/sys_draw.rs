use crate::scene::scene_one::components::{
    AnimFrame, Blink, Enemy, EnemySpriteDefs, EnemySprites, Physics, Player, PlayerSpriteDefs,
    Sprite, Sprites,
};
use crate::systems::Draw;
use crate::{Assets, Config, Context, EStore};
use macroquad::prelude::*;
use tiled::TiledMap;

pub struct DrawSys {
    store: &'static EStore,
    cfg: &'static Config,
    map: Option<TiledMap>,
    sprites: Sprites,
    enemy_sprites: EnemySprites,
}

impl DrawSys {
    pub fn new(
        store: &'static EStore,
        cfg: &'static Config,
        map: Option<TiledMap>,
        assets: &'static Assets,
    ) -> Self {
        let player_defs = store
            .first::<PlayerSpriteDefs>()
            .expect("player sprite defs missing from store");
        let enemy_defs = store
            .first::<EnemySpriteDefs>()
            .expect("enemy sprite defs missing from store");
        let sprites = Sprites {
            idle: Sprite::from_def(&player_defs.idle, assets),
            run: Sprite::from_def(&player_defs.run, assets),
            jump: Sprite::from_def(&player_defs.jump, assets),
            hurt: Sprite::from_def(&player_defs.hurt, assets),
        };
        let enemy_sprites = EnemySprites {
            idle: Sprite::from_def(&enemy_defs.idle, assets),
            walk: Sprite::from_def(&enemy_defs.walk, assets),
            fall: Sprite::from_def(&enemy_defs.fall, assets),
        };
        Self {
            store,
            cfg,
            map,
            sprites,
            enemy_sprites,
        }
    }

    fn draw_tiles(&self, ctx: &Context) {
        let Some(map) = &self.map else {
            return;
        };
        let tw = map.tile_size.0 as f32;
        let th = map.tile_size.1 as f32;
        let view = Rect::new(
            ctx.cam_pan.x,
            ctx.cam_pan.y,
            self.cfg.v_width,
            self.cfg.v_height,
        );
        for section in map.get_sections(view) {
            let cols = (section.bounds.w / tw).round() as usize;
            for (i, tile) in section.tiles.iter().enumerate() {
                let Some(tile) = tile else {
                    continue;
                };
                let col = i % cols;
                let row = i / cols;
                let dx = section.bounds.x + col as f32 * tw;
                let dy = section.bounds.y + row as f32 * th;
                draw_texture_ex(
                    &tile.texture,
                    dx,
                    dy,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(tw, th)),
                        source: Some(tile.source),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_player(&self) {
        let Some(player) = self.store.first::<Player>() else {
            return;
        };
        if let Some(blink) = self.store.get_child::<Blink>(&player) {
            if blink.active && !blink.visible {
                return;
            }
        }
        let Some(frame) = self.store.get_child::<AnimFrame>(&player) else {
            return;
        };
        let Some(phys) = self.store.get_child::<Physics>(&player) else {
            return;
        };
        let sprite = self.sprites.for_anim(player.anim);
        let source = Rect::new(
            (sprite.first_frame + frame.frame) as f32 * sprite.frame_w,
            0.0,
            sprite.frame_w,
            sprite.frame_h,
        );
        let draw_x = player.pos.x + (phys.sprite_size - sprite.frame_w) * 0.5;
        let draw_y = player.pos.y + phys.sprite_size - sprite.frame_h;
        draw_texture_ex(
            &sprite.texture,
            draw_x,
            draw_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(sprite.frame_w, sprite.frame_h)),
                source: Some(source),
                flip_x: player.facing < 0.0,
                ..Default::default()
            },
        );
    }

    fn draw_enemies(&self) {
        let ids = self.store.ids::<Enemy>();
        for id in ids {
            let Some(enemy) = self.store.get_by_id::<Enemy>(id) else {
                continue;
            };
            let Some(frame) = self.store.get_child::<AnimFrame>(&enemy) else {
                continue;
            };
            let Some(phys) = self.store.get_child::<Physics>(&enemy) else {
                continue;
            };
            let sprite = self.enemy_sprites.for_anim(enemy.anim);
            let source = Rect::new(
                (sprite.first_frame + frame.frame) as f32 * sprite.frame_w,
                0.0,
                sprite.frame_w,
                sprite.frame_h,
            );
            let draw_x = enemy.pos.x + (phys.sprite_size - sprite.frame_w) * 0.5;
            let draw_y = enemy.pos.y + phys.sprite_size - sprite.frame_h;
            draw_texture_ex(
                &sprite.texture,
                draw_x,
                draw_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sprite.frame_w, sprite.frame_h)),
                    source: Some(source),
                    flip_x: enemy.facing < 0.0,
                    ..Default::default()
                },
            );
        }
    }
}

impl Draw for DrawSys {
    fn draw(&self, ctx: &Context) {
        draw_rectangle(
            0.0,
            0.0,
            self.cfg.v_width,
            self.cfg.v_height,
            Color::new(0.05, 0.05, 0.1, 1.0),
        );
        self.draw_tiles(ctx);
        self.draw_enemies();
        self.draw_player();
    }
}
