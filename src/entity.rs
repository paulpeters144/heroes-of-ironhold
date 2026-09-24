// Entity sub-modules that expose crate-shared constants stay `pub(crate)`
// (knight/enemy/peon frame constants and factory size constants are used by scene
// systems). Pure-type modules are private; their types are re-exported here.

pub(crate) mod enemy;
pub(crate) mod factory_enemy;
pub(crate) mod factory_hero;
pub(crate) mod factory_peon;
pub(crate) mod knight;
pub(crate) mod peon;

mod animation;
mod area_rect;
mod attack_rect;
mod collision_circle;
mod collision_rect;
mod consecration_aura;
mod dash;
mod drawable;
mod factory_skills;
mod factory_wall;
mod floating_text;
mod health_bar;
mod hero;
mod impact_frame;
mod player;
mod procedural_drawable;
mod rest_node;
mod skills;
mod spawn_zone;
mod static_image;

// Barrel re-exports — the canonical import path for entity types
pub use animation::Animation;
pub use area_rect::AreaRect;
pub use attack_rect::AttackRect;
pub use collision_circle::{CollisionCircle, CollisionGroup};
pub use collision_rect::CollisionRect;
pub use consecration_aura::{load_aura_material, ConsecrationAura};
pub use dash::{Dash, DashCfg};
pub use drawable::Drawable;
pub use enemy::{EnemyStats, RamHead};
pub use factory_enemy::{EnemyFactory, RamHeadCfg};
pub use factory_hero::{HeroFactory, KnightCfg, KnightParts};
pub use factory_peon::{PeonCfg, PeonFactory, PeonParts};
pub use factory_skills::{SkillSlotCfg, SkillSlotParts, SkillsFactory, SkillsParts};
pub use factory_wall::{WallCfg, WallFactory};
pub use floating_text::FloatingText;
pub use health_bar::HealthBar;
pub use hero::{CoreStat, HeroStats};
pub use impact_frame::ImpactFrame;
pub use knight::{
    Consecration, Effect, EffectKind, FrameOffsets, GuardianShield, Knight, KnightLock, Shield,
    Sword,
};
pub use peon::{Peon, PeonStats};
pub use player::{PlayerFactory, PlayerOne, PlayerTwo};
pub use procedural_drawable::{
    ConsecrationData, ProceduralDrawable, ProceduralEffect, RuneShard, Spark,
};
pub use rest_node::RestNode;
pub use skills::{LastUsedSkill, Skill, SkillDirection, SkillIcon, SkillIconKind, SkillsWidget};
pub use spawn_zone::{PeonSpawnZone, RamHeadSpawnZone};
pub use static_image::StaticImage;
