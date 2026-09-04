use crate::GameFont;
use macroquad::prelude::*;

pub(super) enum Cmd {
    Solid {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        radius: f32,
    },
    Outlined {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Color,
        border: Color,
        thickness: f32,
        radius: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font: GameFont,
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        texture: Texture2D,
    },
}
