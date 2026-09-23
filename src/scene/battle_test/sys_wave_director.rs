use crate::entity::{EnemyStats, Knight, PlayerOne, RamHead, RestNode};
use crate::events::{SpawnPeonSquadEvent, SpawnRamHeadEvent};
use crate::prelude::*;
use macroquad::prelude::Vec2;
use std::rc::Rc;

/// A single burst of ram head spawns plus the quiet gap that follows.
#[derive(Clone, Debug)]
pub struct Wave {
    pub at_x: f32,
    pub spawn_count: usize,
    pub spawn_interval: f32,
    pub gap_after: f32,
    pub peon_squad: usize,
}

/// The ordered waves plus the global live-enemy cap.
#[derive(Clone, Debug)]
pub struct WaveSchedule {
    pub waves: Vec<Wave>,
    pub max_alive: usize,
}

/// Where the director is within the current wave: waiting for the knight to
/// reach the gate, firing the burst, or resting in the lull after it.
enum DirectorPhase {
    Waiting,
    Burst,
    Gap,
}

pub struct WaveDirectorSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    schedule: WaveSchedule,
    phase: DirectorPhase,
    wave_index: usize,
    burst_remaining: usize,
    burst_timer: f32,
    gap_timer: f32,
    peon_rallied: bool,
    done: bool,
}

impl WaveDirectorSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, schedule: WaveSchedule) -> Self {
        Self {
            store,
            bus,
            schedule,
            phase: DirectorPhase::Waiting,
            wave_index: 0,
            burst_remaining: 0,
            burst_timer: 0.0,
            gap_timer: 0.0,
            peon_rallied: false,
            done: false,
        }
    }

    fn knight_pos(&self) -> Option<Vec2> {
        self.store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.position)
    }

    fn live_ram_heads(&self) -> usize {
        self.store
            .all::<RamHead>()
            .filter(|enemy| {
                self.store
                    .get_child::<EnemyStats>(enemy)
                    .map(|s| s.hp > 0)
                    .unwrap_or(true)
            })
            .count()
    }

    fn in_rest_node(&self, pos: Vec2) -> bool {
        self.store.all::<RestNode>().any(|node| {
            let r = node.rect;
            pos.x >= r.x && pos.x <= r.x + r.w && pos.y >= r.y && pos.y <= r.y + r.h
        })
    }
}

impl System for WaveDirectorSystem {
    fn update(&mut self, ctx: &mut Context) {
        if self.done {
            return;
        }

        let Some(pos) = self.knight_pos() else {
            return;
        };

        match self.phase {
            DirectorPhase::Waiting => {
                let Some(wave) = self.schedule.waves.get(self.wave_index) else {
                    self.done = true;
                    return;
                };
                if pos.x < wave.at_x {
                    return;
                }
                if self.in_rest_node(pos) {
                    return;
                }
                if self.live_ram_heads() >= self.schedule.max_alive {
                    return;
                }
                self.burst_remaining = wave.spawn_count;
                self.burst_timer = 0.0;
                self.phase = DirectorPhase::Burst;
            }
            DirectorPhase::Burst => {
                let Some(wave) = self.schedule.waves.get(self.wave_index) else {
                    self.done = true;
                    return;
                };
                if self.in_rest_node(pos) {
                    return;
                }
                self.burst_timer -= ctx.dt;
                while self.burst_timer <= 0.0 && self.burst_remaining > 0 {
                    self.bus.fire(&SpawnRamHeadEvent { count: 1 });
                    self.burst_remaining -= 1;
                    self.burst_timer += wave.spawn_interval;
                }
                if self.burst_remaining == 0 {
                    self.gap_timer = wave.gap_after;
                    self.peon_rallied = false;
                    self.phase = DirectorPhase::Gap;
                }
            }
            DirectorPhase::Gap => {
                let Some(wave) = self.schedule.waves.get(self.wave_index) else {
                    self.done = true;
                    return;
                };
                if !self.peon_rallied && wave.peon_squad > 0 {
                    self.bus.fire(&SpawnPeonSquadEvent {
                        count: wave.peon_squad,
                    });
                    self.peon_rallied = true;
                }
                self.gap_timer -= ctx.dt;
                if self.gap_timer <= 0.0 {
                    self.wave_index += 1;
                    self.phase = DirectorPhase::Waiting;
                }
            }
        }
    }
}
