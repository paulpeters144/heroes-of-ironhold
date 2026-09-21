// Re-exports the types every system and scene needs, so consumers can
// `use crate::prelude::*;` instead of assembling imports from multiple paths.

// Infrastructure
pub use crate::{Context, EStore, EventBus};
pub use event_bus::SubCollection;

// Core entity types used by nearly every system
pub use crate::entity::{
    Animation, AttackRect, CollisionRect, Drawable, FloatingText, ProceduralDrawable, StaticImage,
};

// System traits
pub use crate::systems::{System, SystemAgg};
