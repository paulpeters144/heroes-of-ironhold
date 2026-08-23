use crate::raw_models::RawLayer;
use crate::tiled_section::{TiledSection, resolve_section};
use crate::tiled_tileset::TiledTileset;

#[derive(Debug, Clone)]
pub struct TiledLayer {
    pub name: String,
    pub sections: Vec<TiledSection>,
}

pub(crate) fn resolve_layer(layer: RawLayer, tilesets: &[TiledTileset]) -> TiledLayer {
    let sections = layer
        .sections
        .into_iter()
        .map(|section| resolve_section(section, tilesets))
        .collect();

    TiledLayer {
        name: layer.name,
        sections,
    }
}
