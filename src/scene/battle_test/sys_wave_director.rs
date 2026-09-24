use crate::entity::{
    EnemyStats, Knight, PeonSpawnZone, PlayerOne, RamHead, RamHeadSpawnZone, RestNode,
};
use crate::events::{SpawnPeonSquadEvent, SpawnRamHeadEvent};
use crate::prelude::*;
use macroquad::prelude::{Rect, Vec2};
use std::rc::Rc;

// Zone geometry, moved here from sys_peon.rs / sys_ramhead_spawner.rs.
const SPAWN_ZONE_Y: f32 = 125.0; // top of both spawn zones
const SPAWN_ZONE_W: f32 = 50.0; // width of both spawn zones
const SPAWN_ZONE_H: f32 = 200.0; // height of both spawn zones
// Both zones are measured from the knight's current x so they land beyond the
// view's left/right edges (the view is 640 wide; ram heads/peons are 64 wide).
const RAM_ZONE_AHEAD: f32 = 480.0; // ram zone distance ahead of the knight
const PEON_ZONE_BEHIND: f32 = 400.0; // peon zone distance behind the knight

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
    map_w: f32,
}

impl WaveDirectorSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, schedule: WaveSchedule, map_w: f32) -> Self {
        let director = Self {
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
            map_w,
        };
        let initial_x = director.knight_pos().map(|p| p.x).unwrap_or(0.0);
        director.set_spawn_zones(initial_x);
        director
    }

    fn set_spawn_zones(&self, gate_x: f32) {
        let (peon_zone, ram_zone) = spawn_zone_rects(gate_x, self.map_w);

        match self.store.first::<PeonSpawnZone>().map(|z| z.entity_ref()) {
            Some(zone_ref) => {
                self.store.update::<PeonSpawnZone, _>(&zone_ref, |z| z.rect = peon_zone);
            }
            None => {
                self.store.add(PeonSpawnZone { rect: peon_zone }, &[]);
            }
        }

        match self
            .store
            .first::<RamHeadSpawnZone>()
            .map(|z| z.entity_ref())
        {
            Some(zone_ref) => {
                self.store
                    .update::<RamHeadSpawnZone, _>(&zone_ref, |z| z.rect = ram_zone);
            }
            None => {
                self.store.add(RamHeadSpawnZone { rect: ram_zone }, &[]);
            }
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

/// Returns (peon_zone, ram_zone) for the knight at `knight_x`: peon zone
/// `PEON_ZONE_BEHIND` behind the knight clamped at x 0, ram zone
/// `RAM_ZONE_AHEAD` ahead clamped at the map's right edge. Pure free function
/// so the clamp cases are unit-testable.
fn spawn_zone_rects(knight_x: f32, map_w: f32) -> (Rect, Rect) {
    let peon_x = (knight_x - PEON_ZONE_BEHIND - SPAWN_ZONE_W).max(0.0);
    let ram_x = (knight_x + RAM_ZONE_AHEAD).min(map_w - SPAWN_ZONE_W);
    (
        Rect::new(peon_x, SPAWN_ZONE_Y, SPAWN_ZONE_W, SPAWN_ZONE_H),
        Rect::new(ram_x, SPAWN_ZONE_Y, SPAWN_ZONE_W, SPAWN_ZONE_H),
    )
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
                self.set_spawn_zones(pos.x);
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

#[cfg(test)]
mod tests {
    use super::*;

    const MAP_W: f32 = 2000.0;

    #[test]
    fn left_edge_clamps_peon_zone_at_map_start() {
        let (peon, ram) = spawn_zone_rects(0.0, MAP_W);
        assert_eq!(peon.x, 0.0);
        assert_eq!(ram.x, RAM_ZONE_AHEAD);
    }

    #[test]
    fn mid_map_offsets_peon_behind_and_ram_ahead() {
        let (peon, ram) = spawn_zone_rects(1000.0, MAP_W);
        assert_eq!(peon.x, 1000.0 - PEON_ZONE_BEHIND - SPAWN_ZONE_W);
        assert_eq!(ram.x, 1000.0 + RAM_ZONE_AHEAD);
    }

    #[test]
    fn near_right_edge_pins_ram_zone_at_map_end() {
        let (_, ram) = spawn_zone_rects(1700.0, MAP_W);
        assert_eq!(ram.x, MAP_W - SPAWN_ZONE_W);
    }
}
