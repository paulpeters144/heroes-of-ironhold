use std::collections::HashMap;
use std::path::Path;

use macroquad::texture::{FilterMode, Image, Texture2D};
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::TiledError;
use crate::raw_models::ParsedTileset;
use crate::tiled_map::{get_attr_string, get_attr_u16, get_attr_u32};

#[derive(Debug, Clone)]
pub struct TiledTileset {
    pub firstgid: u32,
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u32,
    pub image: Image,
    pub texture: Option<Texture2D>,
}

impl TiledTileset {
    pub fn new(
        firstgid: u32,
        tile_width: u16,
        tile_height: u16,
        columns: u32,
        image: Image,
    ) -> Self {
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        TiledTileset {
            firstgid,
            tile_width,
            tile_height,
            columns,
            image,
            texture: Some(texture),
        }
    }
}

struct TsxInfo {
    tile_width: u16,
    tile_height: u16,
    columns: u32,
    image: String,
}

fn parse_tsx(xml: &str) -> Result<TsxInfo, TiledError> {
    let mut reader = Reader::from_str(xml);
    let mut tile_width: u16 = 0;
    let mut tile_height: u16 = 0;
    let mut columns: u32 = 0;
    let mut image = String::new();

    loop {
        match reader
            .read_event()
            .map_err(|e| TiledError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"tileset" => {
                    if let Some(v) = get_attr_u16(&e, b"tilewidth") {
                        tile_width = v;
                    }
                    if let Some(v) = get_attr_u16(&e, b"tileheight") {
                        tile_height = v;
                    }
                    if let Some(v) = get_attr_u32(&e, b"columns") {
                        columns = v;
                    }
                }
                b"image" => {
                    if let Some(src) = get_attr_string(&e, b"source") {
                        image = Path::new(&src)
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| src.clone());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(TsxInfo {
        tile_width,
        tile_height,
        columns,
        image,
    })
}

pub(crate) fn parse_tileset(
    tsx: &str,
    firstgid: u32,
    images: &HashMap<String, Image>,
) -> Result<ParsedTileset, TiledError> {
    let info = parse_tsx(tsx)?;
    let image = images
        .get(&info.image)
        .cloned()
        .unwrap_or_else(Image::empty);

    Ok(ParsedTileset {
        firstgid,
        tile_width: info.tile_width,
        tile_height: info.tile_height,
        columns: info.columns,
        image,
    })
}

pub(crate) fn resolve_tileset(parsed: &ParsedTileset) -> TiledTileset {
    let texture = Texture2D::from_image(&parsed.image);
    texture.set_filter(FilterMode::Nearest);

    TiledTileset {
        firstgid: parsed.firstgid,
        tile_width: parsed.tile_width,
        tile_height: parsed.tile_height,
        columns: parsed.columns,
        image: parsed.image.clone(),
        texture: Some(texture),
    }
}
