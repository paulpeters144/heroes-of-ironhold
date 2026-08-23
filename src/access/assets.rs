use super::ids::{file, font, image, shader, sound, texture, AssetId};
use di_container::{BuildContext, Injectable};
use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub struct Assets {
    pub textures: HashMap<String, Texture2D>,
    pub images: HashMap<String, Image>,
    pub fonts: HashMap<String, Font>,
    pub sounds: HashMap<String, Sound>,
    pub files: HashMap<String, String>,
    pub shaders: HashMap<String, String>,
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

impl Assets {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            images: HashMap::new(),
            fonts: HashMap::new(),
            sounds: HashMap::new(),
            files: HashMap::new(),
            shaders: HashMap::new(),
        }
    }

    pub async fn load_all(&mut self) {
        for (id, path) in texture::TEXTURES {
            self.load_texture(*id, path).await;
        }
        for (id, path) in font::FONTS {
            self.load_font(*id, path).await;
        }
        for (id, path) in sound::SOUNDS {
            self.load_sound(*id, path).await;
        }
        for (id, path) in image::IMAGES {
            self.load_image(*id, path).await;
        }
        for (id, path) in image::hero::scene_one::FILES {
            self.load_image(*id, path).await;
        }
        for (id, path) in file::FILES {
            self.load_file(*id, path).await;
        }
        for (id, path) in shader::SHADERS {
            self.load_shader(*id, path).await;
        }
    }

    pub async fn load_texture(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                self.textures.insert(key, texture);
            }
            Err(err) => warn!("failed to load texture '{}' as '{}': {}", path, key, err),
        }
    }

    pub async fn load_image(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_image(path).await {
            Ok(image) => {
                self.images.insert(key, image);
            }
            Err(err) => warn!("failed to load image '{}' as '{}': {}", path, key, err),
        }
    }

    pub async fn load_font(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_ttf_font(path).await {
            Ok(mut font) => {
                font.set_filter(FilterMode::Nearest);
                self.fonts.insert(key, font);
            }
            Err(err) => warn!("failed to load font '{}' as '{}': {}", path, key, err),
        }
    }

    pub async fn load_sound(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_sound(path).await {
            Ok(sound) => {
                self.sounds.insert(key, sound);
            }
            Err(err) => warn!("failed to load sound '{}' as '{}': {}", path, key, err),
        }
    }

    pub async fn load_file(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_string(path).await {
            Ok(contents) => {
                self.files.insert(key, contents);
            }
            Err(err) => warn!("failed to load file '{}' as '{}': {}", path, key, err),
        }
    }

    pub async fn load_shader(&mut self, id: impl AssetId, path: &str) {
        let key = id.id();
        match load_string(path).await {
            Ok(contents) => {
                self.shaders.insert(key, contents);
            }
            Err(err) => warn!("failed to load shader '{}' as '{}': {}", path, key, err),
        }
    }

    pub fn texture(&self, id: impl AssetId) -> Option<&Texture2D> {
        self.textures.get(&id.id())
    }

    pub fn image(&self, id: impl AssetId) -> Option<&Image> {
        self.images.get(&id.id())
    }

    pub fn texture_from_image(&self, id: impl AssetId) -> Option<Texture2D> {
        let image = self.images.get(&id.id())?;
        let texture = Texture2D::from_image(image);
        texture.set_filter(FilterMode::Nearest);
        Some(texture)
    }

    pub fn font(&self, id: impl AssetId) -> Option<&Font> {
        self.fonts.get(&id.id())
    }

    pub fn sound(&self, id: impl AssetId) -> Option<&Sound> {
        self.sounds.get(&id.id())
    }

    pub fn file(&self, id: impl AssetId) -> Option<&String> {
        self.files.get(&id.id())
    }

    pub fn shader(&self, id: impl AssetId) -> Option<&String> {
        self.shaders.get(&id.id())
    }
}

impl Injectable for Assets {
    fn inject(
        _ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(async {
            let mut assets = Assets::new();
            assets.load_all().await;
            Ok(assets)
        })
    }
}
