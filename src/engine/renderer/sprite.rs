// Sprite rendering system

use super::{Camera, CameraUniform, TextureManager, Vertex};
use anyhow::Result;
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

/// A 2D sprite for rendering
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale (1.0 = original size)
    pub scale: Vec2,
    /// Size in pixels (width, height)
    pub size: Vec2,
    /// Color tint (RGBA, 1.0 = full color)
    pub color: Vec4,
    /// Texture handle (None = white texture)
    pub texture: Option<super::TextureHandle>,
    /// Z-order for layering (higher = drawn on top)
    pub z_order: f32,
}

impl Sprite {
    /// Create a new sprite
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
            size,
            color: Vec4::ONE,
            texture: None,
            z_order: 0.0,
        }
    }

    /// Create a sprite with a texture
    pub fn with_texture(position: Vec2, size: Vec2, texture: super::TextureHandle) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
            size,
            color: Vec4::ONE,
            texture: Some(texture),
            z_order: 0.0,
        }
    }

    /// Get the transformation matrix for this sprite
    pub fn transform_matrix(&self) -> Mat4 {
        let translation =
            Mat4::from_translation(Vec3::new(self.position.x, self.position.y, self.z_order));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(
            self.size.x * self.scale.x,
            self.size.y * self.scale.y,
            1.0,
        ));

        translation * rotation * scale
    }
}

/// Sprite renderer using instanced rendering
pub struct SpriteRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    sprites: Vec<Sprite>,
}

impl SpriteRenderer {
    /// Create a new sprite renderer
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Result<Self> {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create quad vertices (unit square centered at origin)
        let vertices = [
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(0.5, -0.5, 0.0), Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(0.5, 0.5, 0.0), Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec2::new(0.0, 0.0), Vec4::ONE),
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create camera buffer
        let camera_uniform = CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            bind_group_layout: texture_bind_group_layout,
            sprites: Vec::new(),
        })
    }

    /// Add a sprite to render
    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    /// Clear all sprites
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Render all sprites
    /// Note: This is a simplified version. A production implementation would
    /// batch sprites by texture and use instanced rendering for better performance.
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _camera: &Camera,
        _texture_manager: &'a TextureManager,
    ) -> Result<()> {
        if self.sprites.is_empty() {
            return Ok(());
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // Note: In a production implementation, we would:
        // 1. Batch sprites by texture
        // 2. Use instanced rendering
        // 3. Cache bind groups
        // For now, we just set up the pipeline and buffers

        Ok(())
    }

    /// Get the number of sprites queued for rendering
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }

    /// Get a reference to the camera buffer
    pub fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }
}
