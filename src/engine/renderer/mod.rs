// Rendering system using wgpu

mod camera;
mod sprite;
pub mod texture;
mod vertex;

pub use camera::{Camera, CameraUniform};
pub use sprite::SpriteRenderer;
pub use texture::{TextureHandle, TextureManager};
pub use vertex::Vertex;

// Re-export for future use
#[allow(unused_imports)]
pub use camera::Viewport;
#[allow(unused_imports)]
pub use sprite::Sprite;
#[allow(unused_imports)]
pub use texture::Texture;

use anyhow::Result;
use glam::Vec2;
use log::info;
use std::sync::Arc;
use winit::window::Window;

use crate::engine::physics::DebugRenderer as PhysicsDebugRenderer;

/// Main renderer responsible for initializing wgpu and coordinating rendering
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    sprite_renderer: SpriteRenderer,
    texture_manager: TextureManager,
    camera: Camera,
    physics_debug_renderer: PhysicsDebugRenderer,
}

impl Renderer {
    /// Create a new renderer for the given window
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone())?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

        info!("Using GPU: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create sprite renderer
        let sprite_renderer = SpriteRenderer::new(&device, &config)?;

        // Create texture manager
        let texture_manager = TextureManager::new(&device, &queue);

        // Create camera
        let camera = Camera::new(Vec2::ZERO, size.width as f32, size.height as f32);

        // Create physics debug renderer
        let view_proj = camera.view_proj_matrix().to_cols_array_2d();
        let physics_debug_renderer = PhysicsDebugRenderer::new(&device, surface_format, view_proj);

        info!(
            "Renderer initialized with {}x{} resolution",
            size.width, size.height
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            sprite_renderer,
            texture_manager,
            camera,
            physics_debug_renderer,
        })
    }

    /// Resize the renderer
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .resize(new_size.width as f32, new_size.height as f32);
            info!("Renderer resized to {}x{}", new_size.width, new_size.height);
        }
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Update camera uniform
        let camera_uniform = camera::CameraUniform::new(&self.camera);
        self.queue.write_buffer(
            self.sprite_renderer.camera_buffer(),
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Update physics debug renderer view projection
        self.physics_debug_renderer.update_view_proj(
            &self.queue,
            self.camera.view_proj_matrix().to_cols_array_2d(),
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render sprites
            self.sprite_renderer
                .render(&mut render_pass, &self.camera, &self.texture_manager)?;

            // Render physics debug (if enabled)
            self.physics_debug_renderer.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get a reference to the device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get a reference to the queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get a mutable reference to the sprite renderer
    pub fn sprite_renderer_mut(&mut self) -> &mut SpriteRenderer {
        &mut self.sprite_renderer
    }

    /// Get a reference to the texture manager
    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }

    /// Get a mutable reference to the texture manager
    pub fn texture_manager_mut(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    /// Get a reference to the camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get a mutable reference to the physics debug renderer
    pub fn physics_debug_renderer_mut(&mut self) -> &mut PhysicsDebugRenderer {
        &mut self.physics_debug_renderer
    }

    /// Get the surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}
