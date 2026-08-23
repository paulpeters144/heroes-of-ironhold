use macroquad::math::Rect;
use macroquad::texture::Texture2D;

use crate::tiled_tileset::TiledTileset;

#[derive(Debug, Clone)]
pub struct TiledTile {
    pub texture: Texture2D,
    pub source: Rect,
}

pub(crate) fn resolve_tile(tilesets: &[TiledTileset], gid: u32) -> Option<TiledTile> {
    let tileset = tilesets
        .iter()
        .filter(|ts| ts.firstgid <= gid)
        .max_by_key(|ts| ts.firstgid)?;

    let texture = tileset.texture.clone()?;
    let local = gid - tileset.firstgid;
    let columns = tileset.columns.max(1);
    let tw = tileset.tile_width as f32;
    let th = tileset.tile_height as f32;
    let src_x = (local % columns) as f32 * tw;
    let src_y = (local / columns) as f32 * th;

    Some(TiledTile {
        texture,
        source: Rect::new(src_x, src_y, tw, th),
    })
}
