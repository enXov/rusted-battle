// Asset loading functionality

use super::AssetError;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Supported asset types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Sound,
    Font,
}

impl AssetType {
    /// Get the default directory for this asset type
    pub fn default_directory(&self) -> &'static str {
        match self {
            AssetType::Texture => "textures",
            AssetType::Sound => "sounds",
            AssetType::Font => "fonts",
        }
    }

    /// Get supported file extensions for this asset type
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            AssetType::Texture => &["png", "jpg", "jpeg"],
            AssetType::Sound => &["wav", "ogg", "mp3"],
            AssetType::Font => &["ttf", "otf"],
        }
    }
}

/// Asset loader responsible for finding and loading asset files
pub struct AssetLoader {
    base_path: PathBuf,
}

impl AssetLoader {
    /// Create a new asset loader with the given base path
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the full path for an asset
    pub fn resolve_path(&self, asset_type: AssetType, name: &str) -> PathBuf {
        self.base_path
            .join(asset_type.default_directory())
            .join(name)
    }

    /// Load asset bytes from disk
    pub fn load_bytes(&self, asset_type: AssetType, name: &str) -> Result<Vec<u8>> {
        let path = self.resolve_path(asset_type, name);

        if !path.exists() {
            return Err(AssetError::NotFound(path.to_string_lossy().to_string()).into());
        }

        std::fs::read(&path)
            .map_err(|e| AssetError::LoadError(format!("Failed to read {}: {}", name, e)).into())
    }

    /// Check if an asset exists
    pub fn exists(&self, asset_type: AssetType, name: &str) -> bool {
        self.resolve_path(asset_type, name).exists()
    }

    /// List all assets of a given type
    pub fn list_assets(&self, asset_type: AssetType) -> Result<Vec<String>> {
        let dir = self.base_path.join(asset_type.default_directory());

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut assets = Vec::new();
        let extensions = asset_type.extensions();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        if let Some(name) = path.file_name() {
                            assets.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(assets)
    }

    /// Get the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_directories() {
        assert_eq!(AssetType::Texture.default_directory(), "textures");
        assert_eq!(AssetType::Sound.default_directory(), "sounds");
        assert_eq!(AssetType::Font.default_directory(), "fonts");
    }

    #[test]
    fn test_asset_type_extensions() {
        assert!(AssetType::Texture.extensions().contains(&"png"));
        assert!(AssetType::Sound.extensions().contains(&"wav"));
        assert!(AssetType::Font.extensions().contains(&"ttf"));
    }

    #[test]
    fn test_loader_path_resolution() {
        let loader = AssetLoader::new("/game/assets");
        let path = loader.resolve_path(AssetType::Texture, "player.png");

        assert_eq!(path.to_str().unwrap(), "/game/assets/textures/player.png");
    }

    #[test]
    fn test_loader_exists() {
        let loader = AssetLoader::new(".");
        // This will be false unless we happen to have this file
        let _ = loader.exists(AssetType::Texture, "nonexistent.png");
    }
}
