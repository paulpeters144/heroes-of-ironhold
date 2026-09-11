/// Marker component for a combatant's hit-impact silhouette.
///
/// The actual white `*-hit.png` texture lives in a `StaticImage` child so the
/// existing `DrawSystem` renders it whenever `visible = true`.
#[derive(Clone, Debug)]
pub struct ImpactFrame;
