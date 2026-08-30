use super::ids::{AssetId, AssetKind};
use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;

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

impl Drop for Assets {
    fn drop(&mut self) {
        self.textures.clear();
        self.images.clear();
        self.fonts.clear();
        self.sounds.clear();
        self.files.clear();
        self.shaders.clear();
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

    pub async fn preload(&mut self, ids: &[&dyn AssetId]) {
        for id in ids {
            let path = id.path();
            match id.kind() {
                AssetKind::Texture => {
                    if self.textures.contains_key(&path) {
                        continue;
                    }
                    match load_texture(&path).await {
                        Ok(texture) => {
                            texture.set_filter(FilterMode::Nearest);
                            self.textures.insert(path, texture);
                        }
                        Err(err) => {
                            warn!("failed to load texture '{}': {}", path, err);
                        }
                    }
                }
                AssetKind::Image => {
                    if self.images.contains_key(&path) {
                        continue;
                    }
                    match load_image(&path).await {
                        Ok(image) => {
                            self.images.insert(path, image);
                        }
                        Err(err) => {
                            warn!("failed to load image '{}': {}", path, err);
                        }
                    }
                }
                AssetKind::Font => {
                    if self.fonts.contains_key(&path) {
                        continue;
                    }
                    match load_ttf_font(&path).await {
                        Ok(mut font) => {
                            font.set_filter(FilterMode::Nearest);
                            self.fonts.insert(path, font);
                        }
                        Err(err) => {
                            warn!("failed to load font '{}': {}", path, err);
                        }
                    }
                }
                AssetKind::Sound => {
                    if self.sounds.contains_key(&path) {
                        continue;
                    }
                    match load_sound(&path).await {
                        Ok(sound) => {
                            self.sounds.insert(path, sound);
                        }
                        Err(err) => {
                            warn!("failed to load sound '{}': {}", path, err);
                        }
                    }
                }
                AssetKind::File => {
                    if self.files.contains_key(&path) {
                        continue;
                    }
                    match load_string(&path).await {
                        Ok(contents) => {
                            self.files.insert(path, contents);
                        }
                        Err(err) => {
                            warn!("failed to load file '{}': {}", path, err);
                        }
                    }
                }
                AssetKind::Shader => {
                    if self.shaders.contains_key(&path) {
                        continue;
                    }
                    match load_string(&path).await {
                        Ok(contents) => {
                            self.shaders.insert(path, contents);
                        }
                        Err(err) => {
                            warn!("failed to load shader '{}': {}", path, err);
                        }
                    }
                }
            }
        }
    }

    pub fn texture<T: AssetId>(&self, id: T) -> Texture2D {
        let path = id.path();
        self.textures
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("texture '{}' was not preloaded", path))
    }

    pub fn image<T: AssetId>(&self, id: T) -> Image {
        let path = id.path();
        self.images
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("image '{}' was not preloaded", path))
    }

    pub fn font<T: AssetId>(&self, id: T) -> Font {
        let path = id.path();
        self.fonts
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("font '{}' was not preloaded", path))
    }

    pub fn sound<T: AssetId>(&self, id: T) -> Sound {
        let path = id.path();
        self.sounds
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("sound '{}' was not preloaded", path))
    }

    pub fn file<T: AssetId>(&self, id: T) -> String {
        let path = id.path();
        self.files
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("file '{}' was not preloaded", path))
    }

    pub fn shader<T: AssetId>(&self, id: T) -> String {
        let path = id.path();
        self.shaders
            .get(&path)
            .cloned()
            .unwrap_or_else(|| panic!("shader '{}' was not preloaded", path))
    }
}
