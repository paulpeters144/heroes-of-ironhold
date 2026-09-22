use std::collections::HashMap;

use macroquad::math::{Rect, Vec2};
use macroquad::texture::Image;
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::collide_static::CollideStatic;
use crate::error::TiledError;
use crate::raw_models::RawLayer;
use crate::tiled_layer::{TiledLayer, resolve_layer};
use crate::tiled_section::{TiledSection, build_sections};
use crate::tiled_tileset::{TiledTileset, resolve_tileset};

pub struct TiledMapCfg<'a> {
    pub tile_map: String,
    pub tile_sets: HashMap<String, String>,
    pub images: &'a HashMap<String, Image>,
    pub section_size: (u32, u32),
}

#[derive(Clone)]
pub struct TiledMap {
    pub tilesets: Vec<TiledTileset>,
    pub tile_size: (u16, u16),
    pub map_size: (u32, u32),
    pub section_size: (u32, u32),
    pub layers: Vec<TiledLayer>,
    pub collide_statics: Vec<CollideStatic>,
    pub patrol_points: Vec<Vec2>,
}

pub(crate) fn get_attr_u32(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<u32> {
    let val = e.try_get_attribute(name).ok().flatten()?.value.to_vec();
    String::from_utf8(val).ok()?.parse().ok()
}

pub(crate) fn get_attr_u16(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<u16> {
    let val = e.try_get_attribute(name).ok().flatten()?.value.to_vec();
    String::from_utf8(val).ok()?.parse().ok()
}

pub(crate) fn get_attr_f32(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<f32> {
    let val = e.try_get_attribute(name).ok().flatten()?.value.to_vec();
    String::from_utf8(val).ok()?.parse().ok()
}

pub(crate) fn get_attr_string(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    let val = e.try_get_attribute(name).ok().flatten()?.value.to_vec();
    String::from_utf8(val).ok()
}

fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

impl TiledMap {
    pub fn from_config(cfg: TiledMapCfg<'_>) -> Result<Self, TiledError> {
        let mut reader = Reader::from_str(&cfg.tile_map);

        let mut tile_w: u16 = 0;
        let mut tile_h: u16 = 0;
        let mut map_w: u32 = 0;
        let mut map_h: u32 = 0;
        let mut tileset_srcs: Vec<(u32, String)> = Vec::new();
        let mut raw_layers: Vec<RawLayer> = Vec::new();
        let mut collide_statics: Vec<CollideStatic> = Vec::new();
        let mut patrol_points: Vec<Vec2> = Vec::new();
        let mut in_collide_group = false;
        let mut in_patrol_group = false;

        let mut in_data = false;
        let mut csv_buf = String::new();
        let mut current_layer_name = String::new();
        let mut current_layer_w: u32 = 0;
        let mut current_layer_h: u32 = 0;

        loop {
            match reader
                .read_event()
                .map_err(|e| TiledError::Xml(e.to_string()))?
            {
                Event::Eof => break,
                Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                    b"map" => {
                        tile_w = get_attr_u16(&e, b"tilewidth").unwrap_or(0);
                        tile_h = get_attr_u16(&e, b"tileheight").unwrap_or(0);
                        map_w = get_attr_u32(&e, b"width").unwrap_or(0);
                        map_h = get_attr_u32(&e, b"height").unwrap_or(0);
                    }
                    b"tileset" => {
                        let firstgid = get_attr_u32(&e, b"firstgid").unwrap_or(1);
                        let source = get_attr_string(&e, b"source").unwrap_or_default();
                        tileset_srcs.push((firstgid, source));
                    }
                    b"layer" => {
                        current_layer_name = get_attr_string(&e, b"name").unwrap_or_default();
                        current_layer_w = get_attr_u32(&e, b"width").unwrap_or(0);
                        current_layer_h = get_attr_u32(&e, b"height").unwrap_or(0);
                    }
                    b"data" => {
                        let encoding = get_attr_string(&e, b"encoding").unwrap_or_default();
                        if encoding == "csv" {
                            in_data = true;
                            csv_buf.clear();
                        }
                    }
                    b"objectgroup" => {
                        let name = get_attr_string(&e, b"name").unwrap_or_default();
                        in_collide_group = name == "collide" || name == "collisions";
                        in_patrol_group = name == "patrol";
                    }
                    b"object" if in_collide_group => {
                        let x = get_attr_f32(&e, b"x").unwrap_or(0.0);
                        let y = get_attr_f32(&e, b"y").unwrap_or(0.0);
                        if let (Some(width), Some(height)) =
                            (get_attr_f32(&e, b"width"), get_attr_f32(&e, b"height"))
                        {
                            collide_statics.push(CollideStatic(Rect::new(x, y, width, height)));
                        }
                    }
                    b"object" if in_patrol_group => {
                        let x = get_attr_f32(&e, b"x").unwrap_or(0.0);
                        let y = get_attr_f32(&e, b"y").unwrap_or(0.0);
                        patrol_points.push(Vec2::new(x, y));
                    }
                    b"object" => {}
                    _ => {}
                },
                Event::Text(e) if in_data => {
                    if let Ok(text) = e.decode() {
                        csv_buf.push_str(&text);
                    }
                }
                Event::End(e) if e.name().as_ref() == b"data" && in_data => {
                    in_data = false;
                    let tiles: Vec<u32> = csv_buf
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .filter_map(|s| s.parse().ok())
                        .collect();

                    let sections = build_sections(
                        &tiles,
                        current_layer_w,
                        current_layer_h,
                        tile_w as u32,
                        tile_h as u32,
                        cfg.section_size,
                    );

                    raw_layers.push(RawLayer {
                        name: current_layer_name.clone(),
                        sections,
                    });
                }
                Event::End(e) if e.name().as_ref() == b"objectgroup" => {
                    in_collide_group = false;
                    in_patrol_group = false;
                }
                _ => {}
            }
        }

        // Build per-tileset metadata; fall back to TSX tile size if TMX didn't set it.
        let mut tilesets: Vec<TiledTileset> = Vec::new();
        for (firstgid, src) in &tileset_srcs {
            let tsx = cfg
                .tile_sets
                .get(src)
                .map(|s| s.as_str())
                .unwrap_or_default();
            let (tileset, tile_size) = resolve_tileset(tsx, *firstgid, cfg.images)?;

            if tile_w == 0 && tile_size.0 > 0 {
                tile_w = tile_size.0;
                tile_h = tile_size.1;
            }

            tilesets.push(tileset);
        }

        // Resolve raw GIDs into drawable tiles now that tilesets (with textures) exist.
        let layers: Vec<TiledLayer> = raw_layers
            .into_iter()
            .map(|layer| resolve_layer(layer, &tilesets))
            .collect();

        Ok(TiledMap {
            tilesets,
            tile_size: (tile_w, tile_h),
            map_size: (map_w, map_h),
            section_size: cfg.section_size,
            layers,
            collide_statics,
            patrol_points,
        })
    }

    pub fn get_sections(&self, view: Rect) -> Vec<&TiledSection> {
        self.layers
            .iter()
            .flat_map(|layer| layer.sections.iter())
            .filter(|section| rects_overlap(&section.bounds, &view))
            .collect()
    }
}
