use crate::access::assets::Assets;
use crate::font;
use macroquad::prelude::Color;
use macroquad::prelude::Font;
use macroquad::prelude::WHITE;

#[derive(Clone, Copy)]
pub enum FontTag {
    H1,
    H2,
    H3,
    Body,
    Tiny,
}

impl FontTag {
    pub fn size(self) -> u16 {
        match self {
            Self::H1 => 48,
            Self::H2 => 32,
            Self::H3 => 16,
            Self::Body => 16,
            Self::Tiny => 8,
        }
    }

    pub fn font_id(self) -> font::Font {
        match self {
            Self::Tiny => font::Font::Tiny04b03,
            _ => font::Font::Pixellari,
        }
    }
}

pub struct TextStyle {
    pub tag: FontTag,
    pub color: Color,
}

impl TextStyle {
    pub fn new(tag: FontTag) -> Self {
        Self { tag, color: WHITE }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

#[derive(Clone)]
pub struct GameFont {
    pub font: Font,
    pub size: u16,
    pub color: Color,
}

impl Assets {
    pub fn get_font(&self, style: &TextStyle) -> GameFont {
        let font = self.font(style.tag.font_id());
        GameFont {
            font,
            size: style.tag.size(),
            color: style.color,
        }
    }
}
