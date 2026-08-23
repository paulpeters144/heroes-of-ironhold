# Fonts

This document explains how the game registers, resolves, and draws text. Text
rendering goes through a small styling layer (`FontTag` / `TextStyle` /
`GameFont`) on top of macroquad's `load_ttf_font` + `draw_text_ex` path.

## Font Files

Two fonts are shipped in `assets/fonts/`:

| Asset id          | Path                    | Used for                         |
| ----------------- | ----------------------- | -------------------------------- |
| `font::Id::Pixellari` | `fonts/Pixellari.ttf` | Headings, body text (default) |
| `font::Id::Tiny04b03` | `fonts/04b_03.ttf`   | Tiny 8px pixel font            |

They are declared in `src/access/ids.rs`, both in the `font::Id` enum and the
`font::FONTS` manifest that `Assets::load_all` iterates:

```rust
pub enum Id {
    Pixellari,
    Tiny04b03,
}

pub const FONTS: &[(Id, &str)] = &[
    (Id::Pixellari, "fonts/Pixellari.ttf"),
    (Id::Tiny04b03, "fonts/04b_03.ttf"),
];
```

To add a font: drop the `.ttf` in `assets/fonts/`, add an `Id` variant, and add a
`(Id, path)` entry to `font::FONTS`. `Assets` loads them all at startup and
exposes them through `Assets::font(id)`.

## The Styling Layer

Fonts are never referenced by raw size in scenes. Instead you go through three
small types in `src/access/font.rs`.

### `FontTag` — the size ladder

```rust
pub enum FontTag {
    H1,   // 48px
    H2,   // 32px
    H3,   // 16px
    Body, // 16px
    Tiny, // 8px
}
```

`FontTag::size()` returns the pixel size, and `FontTag::font_id()` picks the font
file: `Tiny` maps to `Tiny04b03`, everything else to `Pixellari`.

### `TextStyle` — tag + color

```rust
pub struct TextStyle {
    pub tag: FontTag,
    pub color: Color,
}
```

Use the builder to set color:

```rust
let style = TextStyle::new(FontTag::H1).color(YELLOW);
```

### `GameFont` — the resolved handle

```rust
pub struct GameFont {
    pub font: &'static Font,
    pub size: u16,
    pub color: Color,
}
```

This is the concrete thing scenes store and pass to draw calls.

## Resolving a Style

`Assets::get_font` turns a `TextStyle` into an optional `GameFont`:

```rust
let gf = assets.get_font(&TextStyle::new(FontTag::Body).color(SKYBLUE));
// gf: Option<GameFont>
```

It returns `None` if the underlying font failed to load (asset loading is
failure-tolerant, so the game keeps running without the text).

For raw access without the styling layer (e.g. the opening title, which measures
at an arbitrary scale), use the font directly:

```rust
let font = assets.font(font::Id::Pixellari); // Option<&Font>
```

## Extracting Fonts

Per the project rules, fonts are resolved **once at construction time** (in
`factory()` / `new()`), never inside `update()` or `draw()`. Store the resulting
`GameFont` (or `Option<GameFont>`) on the struct.

Example from `src/scene/font_showcase/scene.rs`:

```rust
pub fn factory(...) -> Box<dyn SceneFactory> {
    Box::new(SceneFn(move || {
        let meta_font = assets.get_font(&TextStyle::new(FontTag::Tiny).color(GRAY));
        let rows = vec![
            Row { label: "Pixellari | H1 | 48px",
                  gf: assets.get_font(&TextStyle::new(FontTag::H1).color(WHITE)) },
            Row { label: "Pixellari | H2 | 32px",
                  gf: assets.get_font(&TextStyle::new(FontTag::H2).color(YELLOW)) },
            // ...
        ];
        Box::new(FontShowcase { /* ..., */ meta_font, rows })
    }))
}
```

## Drawing and Measuring Text

In `draw()`, render with `draw_text_ex` using the stored `GameFont`:

```rust
if let Some(ref gf) = row.gf {
    let dims = measure_text(SAMPLE, Some(gf.font), gf.size, 1.0);
    draw_text_ex(
        SAMPLE,
        x,
        y,
        TextParams {
            font: Some(gf.font),
            font_size: gf.size,
            font_scale: 1.0,
            color: gf.color,
            ..Default::default()
        },
    );
    y += dims.height + 2.0;
}
```

`measure_text` returns `TextDimensions` (`width`, `height`, `offset_y`) so you can
center text or stack lines before drawing.

## Key Code Locations

- `src/access/ids.rs` — `font::Id` and `font::FONTS` manifest
- `src/access/font.rs` — `FontTag`, `TextStyle`, `GameFont`, `Assets::get_font`
- `src/access/assets.rs` — `load_font`, `Assets::font`
- `src/scene/font_showcase/scene.rs` — reference usage of the styling layer
- `src/scene/opening/sys_draw.rs` — direct `Assets::font` usage + centered text
- `src/lib.rs` — re-exports `FontTag`, `GameFont`, `TextStyle`, `font`
