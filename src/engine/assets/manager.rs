// Central asset management system

use super::atlas::TextureAtlas;
use super::hot_reload::HotReloadWatcher;
use super::{AssetError, AssetHandle, AssetId, AssetLoader, AssetType, TextureAsset};
use crate::engine::renderer::texture::Texture;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Central asset manager for the game
///
/// Handles loading, caching, and hot-reloading of all game assets.
pub struct AssetManager {
    /// Asset loader
    loader: AssetLoader,

    /// Hot reload watcher (only in dev mode)
    #[cfg(debug_assertions)]
    hot_reload: HotReloadWatcher,

    /// Loaded textures
    textures: HashMap<AssetId, Texture>,

    /// Path to ID mapping for textures
    texture_paths: HashMap<String, AssetId>,

    /// Texture atlases
    atlases: HashMap<String, TextureAtlas>,
}

impl AssetManager {
    /// Create a new asset manager
    pub fn new<P: AsRef<Path>>(asset_path: P) -> Self {
        Self {
            loader: AssetLoader::new(asset_path),
            #[cfg(debug_assertions)]
            hot_reload: HotReloadWatcher::new(true),
            textures: HashMap::new(),
            texture_paths: HashMap::new(),
            atlases: HashMap::new(),
        }
    }

    /// Load a texture from disk
    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
    ) -> Result<AssetHandle<TextureAsset>> {
        // Check if already loaded
        if let Some(&id) = self.texture_paths.get(name) {
            return Ok(AssetHandle::new(id));
        }

        // Load texture bytes
        let bytes = self.loader.load_bytes(AssetType::Texture, name)?;

        // Create texture
        let texture = Texture::from_bytes(device, queue, &bytes, name)?;

        // Store texture
        let id = AssetId::from_path(name);
        self.textures.insert(id, texture);
        self.texture_paths.insert(name.to_string(), id);

        // Watch for changes in dev mode
        #[cfg(debug_assertions)]
        {
            let path = self.loader.resolve_path(AssetType::Texture, name);
            let _ = self.hot_reload.watch_file(&path);
        }

        Ok(AssetHandle::new(id))
    }

    /// Create a texture from raw bytes
    pub fn load_texture_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        bytes: &[u8],
    ) -> Result<AssetHandle<TextureAsset>> {
        // Check if already loaded
        if let Some(&id) = self.texture_paths.get(name) {
            return Err(AssetError::AlreadyLoaded(name.to_string()).into());
        }

        // Create texture
        let texture = Texture::from_bytes(device, queue, bytes, name)?;

        // Store texture
        let id = AssetId::from_path(name);
        self.textures.insert(id, texture);
        self.texture_paths.insert(name.to_string(), id);

        Ok(AssetHandle::new(id))
    }

    /// Create a solid color texture
    pub fn create_color_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        color: [u8; 4],
    ) -> Result<AssetHandle<TextureAsset>> {
        // Check if already loaded
        if let Some(&id) = self.texture_paths.get(name) {
            return Err(AssetError::AlreadyLoaded(name.to_string()).into());
        }

        // Create texture
        let texture = Texture::from_color(device, queue, color, Some(name))?;

        // Store texture
        let id = AssetId::from_path(name);
        self.textures.insert(id, texture);
        self.texture_paths.insert(name.to_string(), id);

        Ok(AssetHandle::new(id))
    }

    /// Get a texture by handle
    pub fn get_texture(&self, handle: AssetHandle<TextureAsset>) -> Option<&Texture> {
        self.textures.get(&handle.id())
    }

    /// Add a texture atlas
    pub fn add_atlas(&mut self, name: impl Into<String>, atlas: TextureAtlas) {
        self.atlases.insert(name.into(), atlas);
    }

    /// Get a texture atlas by name
    pub fn get_atlas(&self, name: &str) -> Option<&TextureAtlas> {
        self.atlases.get(name)
    }

    /// List all available assets of a given type
    pub fn list_assets(&self, asset_type: AssetType) -> Result<Vec<String>> {
        self.loader.list_assets(asset_type)
    }

    /// Check if an asset exists
    pub fn asset_exists(&self, asset_type: AssetType, name: &str) -> bool {
        self.loader.exists(asset_type, name)
    }

    /// Check for hot-reloaded assets (dev mode only)
    #[cfg(debug_assertions)]
    pub fn check_hot_reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<String> {
        let changed_files = self.hot_reload.check_all();
        let mut reloaded = Vec::new();

        for path in changed_files {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy().to_string();

                // Try to reload the texture
                if let Some(&id) = self.texture_paths.get(&name_str) {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if let Ok(texture) = Texture::from_bytes(device, queue, &bytes, &name_str) {
                            self.textures.insert(id, texture);
                            reloaded.push(name_str);
                        }
                    }
                }
            }
        }

        reloaded
    }

    /// Get statistics about loaded assets
    pub fn stats(&self) -> AssetStats {
        AssetStats {
            texture_count: self.textures.len(),
            atlas_count: self.atlases.len(),
        }
    }

    /// Get the asset loader
    pub fn loader(&self) -> &AssetLoader {
        &self.loader
    }
}

/// Statistics about loaded assets
#[derive(Debug, Clone, Copy)]
pub struct AssetStats {
    pub texture_count: usize,
    pub atlas_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most AssetManager tests require a GPU device which we can't create in unit tests.
    // These tests focus on the logic that doesn't require GPU.

    #[test]
    fn test_asset_stats() {
        let stats = AssetStats {
            texture_count: 5,
            atlas_count: 2,
        };

        assert_eq!(stats.texture_count, 5);
        assert_eq!(stats.atlas_count, 2);
    }
}
