// Type-safe asset handle system

use std::marker::PhantomData;

/// Unique identifier for an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub(crate) u64);

impl AssetId {
    /// Create a new asset ID from a string path
    pub fn from_path(path: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Create an asset ID from a raw u64
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Type-safe handle to a loaded asset
///
/// The `T` parameter ensures handles can only be used with the correct asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle<T> {
    pub(crate) id: AssetId,
    _phantom: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    /// Create a new asset handle
    pub(crate) fn new(id: AssetId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the underlying asset ID
    pub fn id(&self) -> AssetId {
        self.id
    }
}

// Marker types for different asset types
pub struct TextureAsset;
pub struct SoundAsset;
pub struct FontAsset;

/// Convenience type aliases
pub type TextureHandle = AssetHandle<TextureAsset>;
pub type SoundHandle = AssetHandle<SoundAsset>;
pub type FontHandle = AssetHandle<FontAsset>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id_from_path() {
        let id1 = AssetId::from_path("textures/player.png");
        let id2 = AssetId::from_path("textures/player.png");
        let id3 = AssetId::from_path("textures/enemy.png");

        assert_eq!(id1, id2, "Same paths should produce same IDs");
        assert_ne!(id1, id3, "Different paths should produce different IDs");
    }

    #[test]
    fn test_asset_id_roundtrip() {
        let id = AssetId::from_u64(12345);
        assert_eq!(id.as_u64(), 12345);
    }

    #[test]
    fn test_asset_handle_type_safety() {
        let id = AssetId::from_u64(1);
        let texture_handle: AssetHandle<TextureAsset> = AssetHandle::new(id);
        let sound_handle: AssetHandle<SoundAsset> = AssetHandle::new(id);

        // These are different types but have the same underlying ID
        assert_eq!(texture_handle.id(), sound_handle.id());
    }

    #[test]
    fn test_handle_equality() {
        let id = AssetId::from_u64(42);
        let handle1: TextureHandle = AssetHandle::new(id);
        let handle2: TextureHandle = AssetHandle::new(id);

        // Handles with same ID and type should be equal
        assert_eq!(handle1.id(), handle2.id());
    }
}
