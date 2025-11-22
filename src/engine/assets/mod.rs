// Asset management system
//
// Provides centralized loading, caching, and hot-reloading of game assets.

mod atlas;
mod handle;
mod hot_reload;
mod loader;
mod manager;

pub use atlas::{AtlasBuilder, AtlasRegion, TextureAtlas};
pub use handle::{
    AssetHandle, AssetId, FontAsset, FontHandle, SoundAsset, SoundHandle, TextureAsset,
    TextureHandle,
};
pub use loader::{AssetLoader, AssetType};
pub use manager::AssetManager;

#[cfg(debug_assertions)]
pub use hot_reload::HotReloadWatcher;

/// Asset loading errors
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Asset already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Invalid asset type: expected {expected}, got {actual}")]
    InvalidType { expected: String, actual: String },

    #[error("Failed to load asset: {0}")]
    LoadError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_error_display() {
        let err = AssetError::NotFound("test.png".to_string());
        assert_eq!(err.to_string(), "Asset not found: test.png");
    }
}
