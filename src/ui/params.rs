use crate::{Context, GameFont};
use macroquad::prelude::{Color, Texture2D};

pub struct Label<'a> {
    pub ctx: &'a Context,
    pub font: GameFont,
    pub text: &'a str,
    pub x: f32,
    pub y: f32,
    pub max_width: f32,
}

pub struct Solid<'a> {
    pub ctx: &'a Context,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
    pub radius: f32,
}

pub struct Outlined<'a> {
    pub ctx: &'a Context,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fill: Color,
    pub border: Color,
    pub thickness: f32,
    pub radius: f32,
}

pub struct CenteredSolid<'a> {
    pub ctx: &'a Context,
    pub w: f32,
    pub h: f32,
    pub color: Color,
    pub radius: f32,
}

pub struct CenteredOutlined<'a> {
    pub ctx: &'a Context,
    pub w: f32,
    pub h: f32,
    pub fill: Color,
    pub border: Color,
    pub thickness: f32,
    pub radius: f32,
}

pub struct DefaultOutlined<'a> {
    pub ctx: &'a Context,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct DefaultCenteredOutlined<'a> {
    pub ctx: &'a Context,
    pub w: f32,
    pub h: f32,
}

pub struct Image<'a> {
    pub ctx: &'a Context,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub texture: Texture2D,
}
