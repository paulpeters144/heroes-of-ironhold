use macroquad::math::Rect;

/// A raw, unresolved section of tile GIDs before texture resolution.
#[derive(Debug, Clone)]
pub(crate) struct RawSection {
    pub(crate) grid_pos: (u32, u32),
    pub(crate) bounds: Rect,
    pub(crate) tiles: Vec<u32>,
}

/// A raw, unresolved layer before tile resolution.
#[derive(Debug, Clone)]
pub(crate) struct RawLayer {
    pub(crate) name: String,
    pub(crate) sections: Vec<RawSection>,
}
