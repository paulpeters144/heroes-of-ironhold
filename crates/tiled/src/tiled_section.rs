use macroquad::math::Rect;

use crate::raw_models::RawSection;
use crate::tiled_tile::{TiledTile, resolve_tile};
use crate::tiled_tileset::TiledTileset;

#[derive(Debug, Clone)]
pub struct TiledSection {
    /// Grid coordinates of this section (col, row).
    pub grid_pos: (u32, u32),
    /// World-space bounds of this section.
    pub bounds: Rect,
    /// Drawable tiles for this section (flat array, row-major; None = empty tile).
    pub tiles: Vec<Option<TiledTile>>,
}

pub(crate) fn build_sections(
    tiles: &[u32],
    map_cols: u32,
    map_rows: u32,
    tile_w: u32,
    tile_h: u32,
    section_size: (u32, u32),
) -> Vec<RawSection> {
    let (section_w, section_h) = section_size;
    let num_gx = map_cols.div_ceil(section_w);
    let num_gy = map_rows.div_ceil(section_h);
    let mut sections = Vec::with_capacity((num_gx * num_gy) as usize);

    for gy in 0..num_gy {
        for gx in 0..num_gx {
            let col_start = gx * section_w;
            let row_start = gy * section_h;
            let cols = section_w.min(map_cols - col_start);
            let rows = section_h.min(map_rows - row_start);

            let mut section_tiles = Vec::with_capacity((cols * rows) as usize);
            for r in row_start..row_start + rows {
                for c in col_start..col_start + cols {
                    let idx = (r * map_cols + c) as usize;
                    section_tiles.push(tiles[idx]);
                }
            }

            sections.push(RawSection {
                grid_pos: (gx, gy),
                bounds: Rect::new(
                    (col_start * tile_w) as f32,
                    (row_start * tile_h) as f32,
                    (cols * tile_w) as f32,
                    (rows * tile_h) as f32,
                ),
                tiles: section_tiles,
            });
        }
    }

    sections
}

pub(crate) fn resolve_section(section: RawSection, tilesets: &[TiledTileset]) -> TiledSection {
    let tiles = section
        .tiles
        .into_iter()
        .map(|gid| resolve_tile(tilesets, gid))
        .collect();

    TiledSection {
        grid_pos: section.grid_pos,
        bounds: section.bounds,
        tiles,
    }
}
