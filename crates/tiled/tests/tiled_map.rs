use macroquad::math::Rect;
use macroquad::texture::Image;
use std::collections::HashMap;
use tiled::{TiledMap, TiledMapCfg, TiledSection};

fn get_content(name: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/data/{}", manifest, name);
    let data = std::fs::read_to_string(path);
    data.unwrap()
}

fn sample_tmx() -> String {
    get_content("map.tmx")
}

fn sample_tsx() -> String {
    get_content("bg-1.tsx")
}

fn dummy_image(w: u16, h: u16) -> Image {
    Image {
        width: w,
        height: h,
        bytes: vec![0; (w as usize) * (h as usize) * 4],
    }
}

fn make_cfg(section_size: (u32, u32)) -> TiledMapCfg<'static> {
    let images: &'static HashMap<String, Image> = Box::leak(Box::new(
        vec![("bg-1.png".to_string(), dummy_image(448, 240))]
            .into_iter()
            .collect(),
    ));
    TiledMapCfg {
        tile_map: sample_tmx(),
        tile_sets: vec![("bg-1.tsx".to_string(), sample_tsx())]
            .into_iter()
            .collect(),
        images,
        section_size,
    }
}

fn build_map() -> TiledMap {
    TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed")
}

fn section_grid_positions(map: &TiledMap, view: Rect) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = map.get_sections(view).iter().map(|s| s.grid_pos).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------
// from_config tests
// ---------------------------------------------------------------------------

#[macroquad::test]
async fn parse_tile_size() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    assert_eq!(map.tile_size, (16, 16));
}

#[macroquad::test]
async fn parse_tileset_metadata() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    assert_eq!(map.tilesets.len(), 1);
    let ts = &map.tilesets[0];
    assert_eq!(ts.firstgid, 1);
    assert_eq!(ts.tile_width, 16);
    assert_eq!(ts.tile_height, 16);
    assert_eq!(ts.columns, 28);
    assert_eq!(ts.image.width, 448);
    assert_eq!(ts.image.height, 240);
    assert!(ts.texture.is_some());
}

#[macroquad::test]
async fn parse_layer_count() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    assert_eq!(map.layers.len(), 1);
}

#[macroquad::test]
async fn parse_layer_name() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    assert_eq!(map.layers[0].name, "layer-1");
}

#[macroquad::test]
async fn parse_section_count() {
    // 30x20 map with 16x16 sections => ceil(30/16)=2 x ceil(20/16)=2 = 4
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    let total: usize = map.layers.iter().map(|l| l.sections.len()).sum();
    assert_eq!(total, 4);
}

#[macroquad::test]
async fn parse_section_grid_positions() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    let mut positions: Vec<(u32, u32)> =
        map.layers[0].sections.iter().map(|s| s.grid_pos).collect();
    positions.sort();
    assert_eq!(positions, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
}

#[macroquad::test]
async fn parse_section_bounds() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    let mut sections: Vec<&TiledSection> = map.layers[0].sections.iter().collect();
    sections.sort_by_key(|a| a.grid_pos);

    // (0,0): 16x16 tiles * 16px = 256x256
    let s00 = sections.iter().find(|s| s.grid_pos == (0, 0)).unwrap();
    assert_eq!(s00.bounds.x, 0.0);
    assert_eq!(s00.bounds.y, 0.0);
    assert_eq!(s00.bounds.w, 256.0);
    assert_eq!(s00.bounds.h, 256.0);

    // (1,0): remaining 14 columns * 16px = 224, 16 rows * 16px = 256
    let s10 = sections.iter().find(|s| s.grid_pos == (1, 0)).unwrap();
    assert_eq!(s10.bounds.x, 256.0);
    assert_eq!(s10.bounds.y, 0.0);
    assert_eq!(s10.bounds.w, 224.0);
    assert_eq!(s10.bounds.h, 256.0);
}

#[macroquad::test]
async fn parse_section_tiles_not_empty() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    let has_nonzero = map.layers[0]
        .sections
        .iter()
        .any(|s| s.tiles.iter().any(|t| t.is_some()));
    assert!(
        has_nonzero,
        "expected at least one section with non-zero tiles"
    );
}

#[macroquad::test]
async fn parse_section_tile_count() {
    let map = TiledMap::from_config(make_cfg((16, 16))).expect("from_config failed");
    for section in &map.layers[0].sections {
        let (gx, gy) = section.grid_pos;
        let cols = if gx == 1 { 30 - 16 } else { 16 }; // last col gets 14
        let rows = if gy == 1 { 20 - 16 } else { 16 }; // last row gets 4
        assert_eq!(
            section.tiles.len(),
            cols * rows,
            "section ({},{}) expected {} tiles, got {}",
            gx,
            gy,
            cols * rows,
            section.tiles.len()
        );
    }
}

// ---------------------------------------------------------------------------
// get_sections tests
// ---------------------------------------------------------------------------

#[macroquad::test]
async fn get_sections_full_map() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(0.0, 0.0, 480.0, 320.0));
    assert_eq!(positions, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
}

#[macroquad::test]
async fn get_sections_top_left_quarter() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(0.0, 0.0, 256.0, 256.0));
    assert_eq!(positions, vec![(0, 0)]);
}

#[macroquad::test]
async fn get_sections_top_right_quarter() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(256.0, 0.0, 224.0, 256.0));
    assert_eq!(positions, vec![(1, 0)]);
}

#[macroquad::test]
async fn get_sections_bottom_left_quarter() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(0.0, 256.0, 256.0, 64.0));
    assert_eq!(positions, vec![(0, 1)]);
}

#[macroquad::test]
async fn get_sections_outside_map() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(1000.0, 1000.0, 100.0, 100.0));
    assert!(positions.is_empty());
}

#[macroquad::test]
async fn get_sections_partial_overlap() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(200.0, 200.0, 200.0, 200.0));
    assert_eq!(positions, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
}

#[macroquad::test]
async fn get_sections_left_half() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(0.0, 0.0, 256.0, 320.0));
    assert_eq!(positions, vec![(0, 0), (0, 1)]);
}

#[macroquad::test]
async fn get_sections_right_half() {
    let map = build_map();
    let positions = section_grid_positions(&map, Rect::new(256.0, 0.0, 224.0, 320.0));
    assert_eq!(positions, vec![(1, 0), (1, 1)]);
}
