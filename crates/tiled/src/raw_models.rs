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

/// Parsed tileset metadata without texture (no graphics context required).
#[derive(Debug, Clone)]
pub struct ParsedTileset {
    pub firstgid: u32,
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u32,
    pub image: macroquad::texture::Image,
}

/// Parsed layer with raw tile GIDs (no graphics context required).
#[derive(Debug, Clone)]
pub struct ParsedLayer {
    pub name: String,
    pub sections: Vec<ParsedSection>,
}

/// Parsed section with raw tile GIDs (no graphics context required).
#[derive(Debug, Clone)]
pub struct ParsedSection {
    pub grid_pos: (u32, u32),
    pub bounds: Rect,
    pub tiles: Vec<u32>,
}

/// Fully parsed map data without textures (no graphics context required).
#[derive(Debug, Clone)]
pub struct ParsedMap {
    pub tilesets: Vec<ParsedTileset>,
    pub tile_size: (u16, u16),
    pub map_size: (u32, u32),
    pub section_size: (u32, u32),
    pub layers: Vec<ParsedLayer>,
    pub collide_statics: Vec<crate::collide_static::CollideStatic>,
    pub patrol_points: Vec<macroquad::math::Vec2>,
}
