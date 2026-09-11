use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use pico_entity_store::prelude::EntityRef;

use crate::entity::enemy::RamHead;
use crate::entity::impact_frame::ImpactFrame;
use crate::entity::knight::Knight;
use crate::events::HitEvent;
use crate::systems::Update;
use crate::{Animation, Context, EStore, EventBus, StaticImage, SubCollection};
use macroquad::prelude::*;

/// Peak knockback displacement; drawn at `knockback * t²`.
pub const KNOCKBACK: f32 = 20.0;

/// Reaction length in seconds.
pub const DURATION: f32 = 0.12;

#[derive(Clone, Debug)]
struct ActiveHit {
    anchor: u64,
    impact_image: u64,
    visuals: Vec<u64>,
    direction: Vec2,
    remaining: f32,
}

#[derive(Clone)]
pub struct HitReactionSystem {
    store: Rc<EStore>,
    active: Rc<RefCell<HashMap<u64, ActiveHit>>>,
    queue: Rc<RefCell<VecDeque<HitEvent>>>,
    _subs: Rc<SubCollection>,
}

impl HitReactionSystem {
    /// Subscribes to `HitEvent`. No `assets` param — the impact image is already stored.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<HitEvent>(&bus, move |event: &HitEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            store,
            active: Rc::new(RefCell::new(HashMap::new())),
            queue,
            _subs: subs,
        }
    }

    /// The victim's body anchor `Animation`, whether it's a knight or ram head.
    fn victim_anchor(&self, victim: u64) -> Option<EntityRef> {
        self.store
            .get_by_id::<Knight>(victim)
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|anim| anim.entity_ref())
            .or_else(|| {
                self.store
                    .get_by_id::<RamHead>(victim)
                    .and_then(|ram| self.store.get_child::<Animation>(&ram))
                    .map(|anim| anim.entity_ref())
            })
    }

    /// The victim's impact image (`ImpactFrame` -> `StaticImage`).
    fn victim_impact(&self, victim: u64) -> Option<EntityRef> {
        self.store
            .get_by_id::<Knight>(victim)
            .and_then(|knight| self.store.get_child::<ImpactFrame>(&knight))
            .and_then(|frame| self.store.get_child::<StaticImage>(&frame))
            .map(|image| image.entity_ref())
            .or_else(|| {
                self.store
                    .get_by_id::<RamHead>(victim)
                    .and_then(|ram| self.store.get_child::<ImpactFrame>(&ram))
                    .and_then(|frame| self.store.get_child::<StaticImage>(&frame))
                    .map(|image| image.entity_ref())
            })
    }

    /// The victim's drawable descendants, except the impact image, to hide during a hit.
    fn victim_visuals(&self, victim: u64, impact_id: u64) -> Vec<u64> {
        let descendants: Vec<EntityRef> = if let Some(knight) = self.store.get_by_id::<Knight>(victim)
        {
            self.store.descendants(&knight)
        } else if let Some(ram) = self.store.get_by_id::<RamHead>(victim) {
            self.store.descendants(&ram)
        } else {
            return Vec::new();
        };

        descendants
            .into_iter()
            .filter_map(|e| {
                if e.id() == impact_id {
                    return None;
                }
                if self.store.get_by_id::<Animation>(e.id()).is_some()
                    || self.store.get_by_id::<StaticImage>(e.id()).is_some()
                {
                    Some(e.id())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Knockback direction: from the attacker toward the victim.
    fn knockback_direction(&self, attacker: u64, victim_anchor: &EntityRef) -> Vec2 {
        let victim_center = self
            .store
            .get_by_id::<Animation>(victim_anchor.id())
            .map(|anim| anim.rect().center());
        let attacker_center = self
            .store
            .get_by_id::<Knight>(attacker)
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|anim| anim.rect().center())
            .or_else(|| {
                self.store
                    .get_by_id::<RamHead>(attacker)
                    .and_then(|ram| self.store.get_child::<Animation>(&ram))
                    .map(|anim| anim.rect().center())
            });

        match (victim_center, attacker_center) {
            (Some(victim), Some(attacker)) => {
                let delta = victim - attacker;
                if delta.length_squared() > 0.0 {
                    delta.normalize()
                } else {
                    Vec2::ZERO
                }
            }
            _ => Vec2::ZERO,
        }
    }

    /// Advance every active reaction's knockback, moving its anchor, and restore
    /// victims whose reaction just finished.
    fn advance_knockback(&self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let mut cleared: Vec<ActiveHit> = Vec::new();
        {
            let mut active = self.active.borrow_mut();
            for hit in active.values_mut() {
                // Progress s(u) = knockback * (2u - u²) over DURATION, so the anchor
                // eases back to rest exactly when the reaction ends.
                let rem0 = hit.remaining.max(0.0);
                hit.remaining -= dt;
                let rem1 = hit.remaining.max(0.0);
                let dur = DURATION.max(f32::EPSILON);
                let u0 = ((dur - rem0) / dur).clamp(0.0, 1.0);
                let u1 = ((dur - rem1) / dur).clamp(0.0, 1.0);
                let step = KNOCKBACK * ((2.0 * u1 - u1 * u1) - (2.0 * u0 - u0 * u0));
                let delta = hit.direction * step;
                if delta.length_squared() > 0.0 {
                    if let Some(anim_ref) = self
                        .store
                        .get_by_id::<Animation>(hit.anchor)
                        .map(|anim| anim.entity_ref())
                    {
                        self.store
                            .update::<Animation, _>(&anim_ref, |anim| anim.position += delta);
                    }
                }
            }
            active.retain(|_, hit| {
                if hit.remaining <= 0.0 {
                    cleared.push(hit.clone());
                    false
                } else {
                    true
                }
            });
        }

        for hit in &cleared {
            if let Some(anchor_ref) = self
                .store
                .get_by_id::<Animation>(hit.anchor)
                .map(|anchor| anchor.entity_ref())
            {
                self.store
                    .update::<Animation, _>(&anchor_ref, |anchor| anchor.visible = true);
            }
            if let Some(image_ref) = self
                .store
                .get_by_id::<StaticImage>(hit.impact_image)
                .map(|image| image.entity_ref())
            {
                self.store
                    .update::<StaticImage, _>(&image_ref, |image| image.visible = false);
            }
        }
    }

    /// Hide victim visuals and sync/show the impact image for every active reaction.
    fn sync_visuals(&self) {
        for hit in self.active.borrow().values() {
            for &id in &hit.visuals {
                if let Some(anim_ref) = self
                    .store
                    .get_by_id::<Animation>(id)
                    .map(|anim| anim.entity_ref())
                {
                    self.store
                        .update::<Animation, _>(&anim_ref, |anim| anim.visible = false);
                } else if let Some(image_ref) = self
                    .store
                    .get_by_id::<StaticImage>(id)
                    .map(|image| image.entity_ref())
                {
                    self.store
                        .update::<StaticImage, _>(&image_ref, |image| image.visible = false);
                }
            }

            let Some((position, flip_x)) = self
                .store
                .get_by_id::<Animation>(hit.anchor)
                .map(|anim| (anim.position, anim.flip_x))
            else {
                continue;
            };
            if let Some(image_ref) = self
                .store
                .get_by_id::<StaticImage>(hit.impact_image)
                .map(|image| image.entity_ref())
            {
                self.store.update::<StaticImage, _>(&image_ref, |image| {
                    image.position = position;
                    image.flip_x = flip_x;
                    image.visible = true;
                });
            }
        }
    }
}

impl Update for HitReactionSystem {
    fn update(&mut self, ctx: &mut Context) {
        while let Some(event) = self.queue.borrow_mut().pop_front() {
            let anchor = self.victim_anchor(event.victim);
            let impact = self.victim_impact(event.victim);
            let (Some(anchor), Some(impact)) = (anchor, impact) else {
                continue;
            };

            let visuals = self.victim_visuals(event.victim, impact.id());
            let direction = self.knockback_direction(event.attacker, &anchor);

            self.active.borrow_mut().insert(
                event.victim,
                ActiveHit {
                    anchor: anchor.id(),
                    impact_image: impact.id(),
                    visuals,
                    direction,
                    remaining: DURATION,
                },
            );
        }

        self.advance_knockback(ctx.dt);
        self.sync_visuals();
    }
}
