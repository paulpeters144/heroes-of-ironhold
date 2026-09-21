use crate::entity::knight::{Consecration, Knight, KnightLock, IDLE_FRAME, THRUST_FRAME};
use crate::entity::{
    load_aura_material, AreaRect, ConsecrationAura, ConsecrationData, PlayerOne,
    ProceduralDrawable, ProceduralEffect, RuneShard, SkillIconKind, Spark,
};
use crate::events::SkillCastEvent;
use crate::prelude::*;
use crate::Assets;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const AREA_WIDTH: f32 = 140.0;
const AREA_HEIGHT: f32 = 90.0;
const RECT_WIDTH: f32 = AREA_WIDTH * 0.96;
const RECT_HEIGHT: f32 = AREA_WIDTH * 0.96 * 0.65;
const AREA_SECS: f32 = 24.0;
const LOCK_SECS: f32 = 0.25;

const SHARD_SPAWN_RATE: f32 = 6.0;

pub struct ConsecrationSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    area_ref: Option<EntityRef>,
    marker_ref: Option<EntityRef>,
    ground_ref: Option<EntityRef>,
    aura_ref: Option<EntityRef>,
    active: bool,
    center: Vec2,
    area_life: f32,
    lock_remaining: f32,
    sparks: Vec<Spark>,
    shards: Vec<RuneShard>,
    shard_timer: f32,
    material: Option<Material>,
}

impl ConsecrationSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<SkillCastEvent>(&bus, move |event: &SkillCastEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            store,
            queue,
            _subs: subs,
            area_ref: None,
            marker_ref: None,
            ground_ref: None,
            aura_ref: None,
            active: false,
            center: Vec2::ZERO,
            area_life: 0.0,
            lock_remaining: 0.0,
            sparks: Vec::new(),
            shards: Vec::new(),
            shard_timer: 0.0,
            material: load_aura_material(assets),
        }
    }

    fn begin_cast(&mut self, event: &SkillCastEvent) {
        if event.kind != SkillIconKind::Consecration {
            return;
        }
        if self.active || self.lock_remaining > 0.0 {
            return;
        }

        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        if self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .is_some()
        {
            return;
        }

        let Some(feet) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| {
                let r = a.rect();
                vec2(r.center().x, r.y + r.h)
            })
        else {
            return;
        };

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = THRUST_FRAME;
            });
        }

        let knight = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .expect("knight alive during cast");
        self.store
            .add(knight, &[KnightLock { by: "Consecration" }.into_child()]);

        self.center = feet;
        self.active = true;
        self.area_life = 0.0;
        self.lock_remaining = LOCK_SECS;
        self.spawn_area(feet);
        self.spawn_burst_sparks(feet);
        self.spawn_ground_component(feet, knight_ref.id());
        self.spawn_aura_component(knight_ref.id());
    }

    fn release_lock(&mut self) {
        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        if let Some(lock_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .map(|l| l.entity_ref())
        {
            self.store.remove(&[lock_ref]);
        }

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = IDLE_FRAME;
            });
        }
    }

    fn spawn_area(&mut self, center: Vec2) {
        let rect = Rect::new(
            center.x - RECT_WIDTH * 0.5,
            center.y - RECT_HEIGHT * 0.5,
            RECT_WIDTH,
            RECT_HEIGHT,
        );
        self.store.add(AreaRect { rect }, &[]);
        self.area_ref = self.store.all::<AreaRect>().map(|a| a.entity_ref()).last();

        self.store.add(Consecration { rect }, &[]);
        self.marker_ref = self
            .store
            .all::<Consecration>()
            .map(|c| c.entity_ref())
            .last();
    }

    fn spawn_ground_component(&mut self, center: Vec2, knight_id: u64) {
        let knight_z = self
            .store
            .get_by_id::<Knight>(knight_id)
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.z_idx)
            .unwrap_or(0.0);

        let data = ConsecrationData {
            center,
            area_life: 0.0,
            duration: AREA_SECS,
            sparks: Vec::new(),
            shards: Vec::new(),
        };

        let drawable = ProceduralDrawable {
            effect: ProceduralEffect::Consecration(data),
            z_idx: knight_z - 10.0,
            visible: true,
        };
        self.store.add(drawable, &[]);
        self.ground_ref = self
            .store
            .all::<ProceduralDrawable>()
            .map(|g| g.entity_ref())
            .last();
    }

    fn spawn_aura_component(&mut self, knight_id: u64) {
        let knight_z = self
            .store
            .get_by_id::<Knight>(knight_id)
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.z_idx)
            .unwrap_or(0.0);

        let aura = ConsecrationAura {
            knight_id,
            z_idx: knight_z + 10.0,
            visible: true,
        };
        self.store.add(aura, &[]);
        self.aura_ref = self
            .store
            .all::<ConsecrationAura>()
            .map(|a| a.entity_ref())
            .last();
    }

    fn remove_area(&mut self) {
        if let Some(area_ref) = self.area_ref.take() {
            self.store.remove(&[area_ref]);
        }
        if let Some(marker_ref) = self.marker_ref.take() {
            self.store.remove(&[marker_ref]);
        }
        if let Some(ground_ref) = self.ground_ref.take() {
            self.store.remove(&[ground_ref]);
        }
        if let Some(aura_ref) = self.aura_ref.take() {
            self.store.remove(&[aura_ref]);
        }
    }

    fn spawn_burst_sparks(&mut self, center: Vec2) {
        self.sparks.clear();
        for i in 0..12 {
            let angle = i as f32 * std::f32::consts::TAU / 12.0 + gen_range(-0.1, 0.1);
            let speed = gen_range(50.0, 100.0);
            let size = gen_range(1.5, 3.0);
            self.sparks.push(Spark {
                pos: center,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                size,
                life: gen_range(0.3, 0.6),
            });
        }
    }

    fn spawn_shard(&mut self) {
        let rx = gen_range(-0.4, 0.4) * AREA_WIDTH;
        let ry = gen_range(-0.3, 0.3) * AREA_HEIGHT;
        let life = gen_range(1.2, 2.0);
        self.shards.push(RuneShard {
            pos: self.center + vec2(rx, ry),
            vel: vec2(gen_range(-4.0, 4.0), gen_range(-30.0, -15.0)),
            size: gen_range(2.0, 4.0),
            rotation: gen_range(0.0, std::f32::consts::TAU),
            rot_speed: gen_range(-3.0, 3.0),
            life,
            max_life: life,
        });
    }

    fn update_ground_component(&self) {
        let Some(ground_ref) = &self.ground_ref else {
            return;
        };
        self.store.update::<ProceduralDrawable, _>(ground_ref, |d| {
            let ProceduralEffect::Consecration(ref mut data) = d.effect;
            data.area_life = self.area_life;
            data.sparks = self.sparks.clone();
            data.shards = self.shards.clone();
        });
    }
}

impl System for ConsecrationSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<SkillCastEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_cast(&event);
        }

        if self.lock_remaining > 0.0 {
            self.lock_remaining -= ctx.dt;
            if self.lock_remaining <= 0.0 {
                self.release_lock();
            }
        }

        if self.active {
            self.area_life += ctx.dt;

            for spark in self.sparks.iter_mut() {
                spark.pos += spark.vel * ctx.dt;
                spark.vel *= (1.0 - 4.0 * ctx.dt).max(0.0);
                spark.life -= ctx.dt;
            }
            self.sparks.retain(|s| s.life > 0.0);

            self.shard_timer -= ctx.dt;
            if self.shard_timer <= 0.0 {
                self.shard_timer = 1.0 / SHARD_SPAWN_RATE;
                self.spawn_shard();
            }
            for shard in self.shards.iter_mut() {
                shard.pos += shard.vel * ctx.dt;
                shard.vel.x += (self.area_life * 3.0 + shard.pos.y * 0.05).sin() * 3.0 * ctx.dt;
                shard.rotation += shard.rot_speed * ctx.dt;
                shard.life -= ctx.dt;
            }
            self.shards.retain(|s| s.life > 0.0);

            self.update_ground_component();

            if self.area_life >= AREA_SECS {
                self.active = false;
                self.area_life = 0.0;
                self.sparks.clear();
                self.shards.clear();
                self.remove_area();
            }
        }
    }

    fn draw(&self, _ctx: &Context) {
        if !self.active {
            return;
        }

        let Some(material) = &self.material else {
            return;
        };

        let knight_feet = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| {
                let r = a.rect();
                vec2(r.center().x, r.y + r.h)
            });

        let Some(knight_feet) = knight_feet else {
            return;
        };

        let half_w = RECT_WIDTH * 0.5;
        let half_h = RECT_HEIGHT * 0.5;
        let inside = knight_feet.x >= self.center.x - half_w
            && knight_feet.x <= self.center.x + half_w
            && knight_feet.y >= self.center.y - half_h
            && knight_feet.y <= self.center.y + half_h;

        if !inside {
            return;
        }

        if let Some(aura) = self.store.first::<ConsecrationAura>() {
            aura.draw_with_material(material, &self.store);
        }
    }
}
