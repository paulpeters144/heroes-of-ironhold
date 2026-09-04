use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub version: u32,
}

impl Default for GameState {
    fn default() -> Self {
        GameState { version: 1 }
    }
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(err) => write!(f, "io error: {err}"),
            SaveError::Json(err) => write!(f, "json error: {err}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::Io(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::Json(err)
    }
}

pub trait GameStateStore {
    fn save(&self, state: &GameState) -> Result<(), SaveError>;
    fn load(&self) -> Result<Option<GameState>, SaveError>;
}

pub struct DiskJsonStore {
    path: PathBuf,
}

impl DiskJsonStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        DiskJsonStore {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Default for DiskJsonStore {
    fn default() -> Self {
        DiskJsonStore::new("saves/save.json")
    }
}

impl GameStateStore for DiskJsonStore {
    fn save(&self, state: &GameState) -> Result<(), SaveError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(state)?;

        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &self.path)?;

        Ok(())
    }

    fn load(&self) -> Result<Option<GameState>, SaveError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&self.path)?;
        Ok(Some(serde_json::from_str(&json)?))
    }
}
