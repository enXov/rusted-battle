// Texture atlas system for efficient sprite batching

use glam::Vec2;

/// A region within a texture atlas
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// Name/ID of the sprite
    pub name: &'static str,

    /// Position in the atlas (pixels)
    pub x: u32,
    pub y: u32,

    /// Size of the region (pixels)
    pub width: u32,
    pub height: u32,

    /// UV coordinates (0.0 to 1.0)
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

impl AtlasRegion {
    /// Create a new atlas region with calculated UV coordinates
    pub fn new(
        name: &'static str,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Self {
        let uv_min = Vec2::new(
            x as f32 / atlas_width as f32,
            y as f32 / atlas_height as f32,
        );
        let uv_max = Vec2::new(
            (x + width) as f32 / atlas_width as f32,
            (y + height) as f32 / atlas_height as f32,
        );

        Self {
            name,
            x,
            y,
            width,
            height,
            uv_min,
            uv_max,
        }
    }
}

/// A texture atlas containing multiple sprites
pub struct TextureAtlas {
    /// Width of the atlas texture
    pub width: u32,

    /// Height of the atlas texture
    pub height: u32,

    /// All regions in this atlas
    regions: Vec<AtlasRegion>,
}

impl TextureAtlas {
    /// Create a new texture atlas
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            regions: Vec::new(),
        }
    }

    /// Add a region to the atlas
    pub fn add_region(&mut self, region: AtlasRegion) {
        self.regions.push(region);
    }

    /// Get a region by name
    pub fn get_region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.iter().find(|r| r.name == name)
    }

    /// Get all regions
    pub fn regions(&self) -> &[AtlasRegion] {
        &self.regions
    }

    /// Get the number of regions
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

/// Builder for creating texture atlases
pub struct AtlasBuilder {
    width: u32,
    height: u32,
    padding: u32,
    current_x: u32,
    current_y: u32,
    row_height: u32,
    regions: Vec<AtlasRegion>,
}

impl AtlasBuilder {
    /// Create a new atlas builder
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            padding: 1,
            current_x: 0,
            current_y: 0,
            row_height: 0,
            regions: Vec::new(),
        }
    }

    /// Set the padding between sprites
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Add a sprite to the atlas
    ///
    /// Returns the region if successful, or None if it doesn't fit
    pub fn add_sprite(
        &mut self,
        name: &'static str,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Option<AtlasRegion> {
        // Check if sprite fits in current row
        if self.current_x + sprite_width > self.width {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.row_height + self.padding;
            self.row_height = 0;
        }

        // Check if sprite fits in atlas at all
        if self.current_y + sprite_height > self.height {
            return None;
        }

        // Create region
        let region = AtlasRegion::new(
            name,
            self.current_x,
            self.current_y,
            sprite_width,
            sprite_height,
            self.width,
            self.height,
        );

        self.regions.push(region);

        // Update position
        self.current_x += sprite_width + self.padding;
        self.row_height = self.row_height.max(sprite_height);

        Some(region)
    }

    /// Build the final atlas
    pub fn build(self) -> TextureAtlas {
        let mut atlas = TextureAtlas::new(self.width, self.height);
        for region in self.regions {
            atlas.add_region(region);
        }
        atlas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_region_uv() {
        let region = AtlasRegion::new("test", 0, 0, 64, 64, 256, 256);

        assert_eq!(region.uv_min, Vec2::new(0.0, 0.0));
        assert_eq!(region.uv_max, Vec2::new(0.25, 0.25));
    }

    #[test]
    fn test_atlas_creation() {
        let mut atlas = TextureAtlas::new(256, 256);
        let region = AtlasRegion::new("sprite1", 0, 0, 32, 32, 256, 256);
        atlas.add_region(region);

        assert_eq!(atlas.region_count(), 1);
        assert!(atlas.get_region("sprite1").is_some());
        assert!(atlas.get_region("nonexistent").is_none());
    }

    #[test]
    fn test_atlas_builder() {
        let mut builder = AtlasBuilder::new(256, 256);

        // Add several sprites
        let sprite1 = builder.add_sprite("sprite1", 64, 64);
        let sprite2 = builder.add_sprite("sprite2", 32, 32);
        let sprite3 = builder.add_sprite("sprite3", 48, 48);

        assert!(sprite1.is_some());
        assert!(sprite2.is_some());
        assert!(sprite3.is_some());

        let atlas = builder.build();
        assert_eq!(atlas.region_count(), 3);
    }

    #[test]
    fn test_atlas_builder_overflow() {
        let mut builder = AtlasBuilder::new(64, 64);

        // Try to add a sprite that's too large
        let sprite = builder.add_sprite("toolarge", 128, 128);
        assert!(sprite.is_none());
    }

    #[test]
    fn test_atlas_builder_wrapping() {
        let mut builder = AtlasBuilder::new(100, 100).with_padding(0);

        // Add sprites that will wrap to next row
        let sprite1 = builder.add_sprite("s1", 60, 30);
        let sprite2 = builder.add_sprite("s2", 60, 30); // Should wrap

        assert!(sprite1.is_some());
        assert!(sprite2.is_some());

        let s1 = sprite1.unwrap();
        let s2 = sprite2.unwrap();

        assert_eq!(s1.y, 0);
        assert_eq!(s2.y, 30); // On next row
    }
}
