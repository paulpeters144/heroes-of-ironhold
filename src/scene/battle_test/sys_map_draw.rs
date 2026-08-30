use crate::systems::Draw;
use crate::Context;
use macroquad::prelude::*;
use tiled::TiledMap;

pub struct MapDrawSystem {
    map: TiledMap,
    view_w: f32,
    view_h: f32,
}

impl MapDrawSystem {
    pub fn new(map: TiledMap, view_w: f32, view_h: f32) -> Self {
        Self {
            map,
            view_w,
            view_h,
        }
    }
}

impl Draw for MapDrawSystem {
    fn draw(&self, ctx: &Context) {
        let view = Rect::new(
            ctx.cam_target.x - self.view_w * 0.5,
            ctx.cam_target.y - self.view_h * 0.5,
            self.view_w,
            self.view_h,
        );

        let cell_w = self.map.tile_size.0 as f32;
        let cell_h = self.map.tile_size.1 as f32;

        for section in self.map.get_sections(view) {
            let cols = (section.bounds.w / cell_w).round() as u32;
            let cols = cols.max(1);

            for (i, tile) in section.tiles.iter().enumerate() {
                if let Some(tile) = tile {
                    let col = (i as u32) % cols;
                    let row = (i as u32) / cols;
                    let x = section.bounds.x + col as f32 * cell_w;
                    let y = section.bounds.y + row as f32 * cell_h;

                    draw_texture_ex(
                        &tile.texture,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(tile.source.w, tile.source.h)),
                            source: Some(tile.source),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}
