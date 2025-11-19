// Texture loading and management system

use anyhow::Result;
use image::GenericImageView;
use std::collections::HashMap;
use std::path::Path;

/// Handle to a loaded texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(usize);

/// A loaded texture with GPU resources
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Create a texture from image bytes
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    /// Create a texture from an image
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            width: dimensions.0,
            height: dimensions.1,
        })
    }

    /// Create a solid color texture (useful for testing)
    pub fn from_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        label: Option<&str>,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &color,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            width: 1,
            height: 1,
        })
    }
}

/// Manages texture loading and caching
pub struct TextureManager {
    textures: Vec<Texture>,
    path_to_handle: HashMap<String, TextureHandle>,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        Self {
            textures: Vec::new(),
            path_to_handle: HashMap::new(),
        }
    }

    /// Load a texture from a file path
    pub fn load_texture<P: AsRef<Path>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
    ) -> Result<TextureHandle> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Check if already loaded
        if let Some(&handle) = self.path_to_handle.get(&path_str) {
            return Ok(handle);
        }

        // Load texture
        let bytes = std::fs::read(&path)?;
        let texture = Texture::from_bytes(device, queue, &bytes, &path_str)?;

        // Store texture
        let handle = TextureHandle(self.textures.len());
        self.textures.push(texture);
        self.path_to_handle.insert(path_str, handle);

        Ok(handle)
    }

    /// Load a texture from bytes
    pub fn load_texture_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<TextureHandle> {
        let texture = Texture::from_bytes(device, queue, bytes, label)?;
        let handle = TextureHandle(self.textures.len());
        self.textures.push(texture);
        Ok(handle)
    }

    /// Create a solid color texture
    pub fn create_color_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        label: &str,
    ) -> Result<TextureHandle> {
        let texture = Texture::from_color(device, queue, color, Some(label))?;
        let handle = TextureHandle(self.textures.len());
        self.textures.push(texture);
        Ok(handle)
    }

    /// Get a texture by handle
    pub fn get(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle.0)
    }

    /// Get the number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
}
