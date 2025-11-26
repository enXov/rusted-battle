// Sprite rendering system

use super::{Camera, CameraUniform, TextureManager, Vertex};
use anyhow::Result;
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

/// UV coordinates for a sprite frame
#[derive(Debug, Clone, Copy)]
pub struct SpriteUV {
    /// Minimum UV (top-left)
    pub min: Vec2,
    /// Maximum UV (bottom-right)
    pub max: Vec2,
}

impl Default for SpriteUV {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        }
    }
}

impl SpriteUV {
    /// Create UV coordinates for the full texture
    pub fn full() -> Self {
        Self::default()
    }

    /// Create UV coordinates for a specific frame in a horizontal sprite sheet
    pub fn from_sprite_sheet(frame: usize, total_frames: usize, flip_horizontal: bool) -> Self {
        let frame_width = 1.0 / total_frames as f32;
        let u_min = frame as f32 * frame_width;
        let u_max = u_min + frame_width;

        if flip_horizontal {
            Self {
                min: Vec2::new(u_max, 0.0),
                max: Vec2::new(u_min, 1.0),
            }
        } else {
            Self {
                min: Vec2::new(u_min, 0.0),
                max: Vec2::new(u_max, 1.0),
            }
        }
    }

    /// Create UV coordinates for a frame in a grid sprite sheet
    pub fn from_grid(
        column: usize,
        row: usize,
        total_columns: usize,
        total_rows: usize,
        flip_horizontal: bool,
    ) -> Self {
        let frame_width = 1.0 / total_columns as f32;
        let frame_height = 1.0 / total_rows as f32;

        let u_min = column as f32 * frame_width;
        let u_max = u_min + frame_width;
        let v_min = row as f32 * frame_height;
        let v_max = v_min + frame_height;

        if flip_horizontal {
            Self {
                min: Vec2::new(u_max, v_min),
                max: Vec2::new(u_min, v_max),
            }
        } else {
            Self {
                min: Vec2::new(u_min, v_min),
                max: Vec2::new(u_max, v_max),
            }
        }
    }
}

/// A 2D sprite for rendering
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale (1.0 = original size)
    pub scale: Vec2,
    /// Size in world units (width, height)
    pub size: Vec2,
    /// Color tint (RGBA, 1.0 = full color)
    pub color: Vec4,
    /// Texture handle (None = white texture)
    pub texture: Option<super::TextureHandle>,
    /// UV coordinates for the sprite (for animation frames)
    pub uv: SpriteUV,
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
            uv: SpriteUV::default(),
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
            uv: SpriteUV::default(),
            z_order: 0.0,
        }
    }

    /// Set UV coordinates (for animation frames)
    pub fn with_uv(mut self, uv: SpriteUV) -> Self {
        self.uv = uv;
        self
    }

    /// Set color tint
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Set z-order
    pub fn with_z_order(mut self, z_order: f32) -> Self {
        self.z_order = z_order;
        self
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

    /// Create vertices for this sprite with proper UV coordinates
    pub fn vertices(&self) -> [Vertex; 4] {
        let transform = self.transform_matrix();

        // Quad corners in local space
        let corners = [
            Vec3::new(-0.5, -0.5, 0.0), // Bottom-left
            Vec3::new(0.5, -0.5, 0.0),  // Bottom-right
            Vec3::new(0.5, 0.5, 0.0),   // Top-right
            Vec3::new(-0.5, 0.5, 0.0),  // Top-left
        ];

        // UV coordinates for each corner
        let uvs = [
            Vec2::new(self.uv.min.x, self.uv.max.y), // Bottom-left
            Vec2::new(self.uv.max.x, self.uv.max.y), // Bottom-right
            Vec2::new(self.uv.max.x, self.uv.min.y), // Top-right
            Vec2::new(self.uv.min.x, self.uv.min.y), // Top-left
        ];

        [
            Vertex::new(transform.transform_point3(corners[0]), uvs[0], self.color),
            Vertex::new(transform.transform_point3(corners[1]), uvs[1], self.color),
            Vertex::new(transform.transform_point3(corners[2]), uvs[2], self.color),
            Vertex::new(transform.transform_point3(corners[3]), uvs[3], self.color),
        ]
    }
}

/// Maximum number of sprites that can be batched in a single draw call
const MAX_SPRITES: usize = 1000;
const VERTICES_PER_SPRITE: usize = 4;
const INDICES_PER_SPRITE: usize = 6;

/// Sprite renderer with batching support
pub struct SpriteRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sprites: Vec<Sprite>,
    // For dynamic vertex updates
    vertex_data: Vec<Vertex>,
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
                cull_mode: None, // Don't cull for 2D sprites
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

        // Create dynamic vertex buffer (can hold MAX_SPRITES sprites)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: (MAX_SPRITES * VERTICES_PER_SPRITE * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create index buffer with pre-computed indices for all sprites
        let mut indices = Vec::with_capacity(MAX_SPRITES * INDICES_PER_SPRITE);
        for i in 0..MAX_SPRITES {
            let base = (i * VERTICES_PER_SPRITE) as u16;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

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
            texture_bind_group_layout,
            sprites: Vec::new(),
            vertex_data: Vec::with_capacity(MAX_SPRITES * VERTICES_PER_SPRITE),
        })
    }

    /// Add a sprite to render this frame
    pub fn add_sprite(&mut self, sprite: Sprite) {
        if self.sprites.len() < MAX_SPRITES {
            self.sprites.push(sprite);
        }
    }

    /// Clear all sprites (call at end of frame)
    pub fn clear(&mut self) {
        self.sprites.clear();
        self.vertex_data.clear();
    }

    /// Get the texture bind group layout (for creating texture bind groups)
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    /// Prepare sprites for rendering (uploads vertex data)
    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        self.vertex_data.clear();

        for sprite in &self.sprites {
            let vertices = sprite.vertices();
            self.vertex_data.extend_from_slice(&vertices);
        }

        if !self.vertex_data.is_empty() {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.vertex_data),
            );
        }
    }

    /// Render all sprites
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _camera: &Camera,
        texture_manager: &'a TextureManager,
    ) -> Result<()> {
        if self.sprites.is_empty() {
            return Ok(());
        }

        // Debug: uncomment to log sprite rendering
        // log::info!("Rendering {} sprites", self.sprites.len());

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // Draw sprites batched by texture
        // For now, simple approach: draw each sprite with its texture
        let mut current_sprite_index = 0;
        let mut draw_count = 0;

        for sprite in &self.sprites {
            // Get texture bind group
            if let Some(texture_handle) = &sprite.texture {
                if let Some(bind_group) = texture_manager.get_bind_group(*texture_handle) {
                    render_pass.set_bind_group(1, bind_group, &[]);

                    let index_start = (current_sprite_index * INDICES_PER_SPRITE) as u32;
                    let index_count = INDICES_PER_SPRITE as u32;

                    render_pass.draw_indexed(index_start..(index_start + index_count), 0, 0..1);
                    draw_count += 1;
                } else {
                    log::warn!("No bind group for texture handle {:?}", texture_handle);
                }
            } else {
                // Use default white texture if available
                if let Some(bind_group) = texture_manager.get_default_bind_group() {
                    render_pass.set_bind_group(1, bind_group, &[]);

                    let index_start = (current_sprite_index * INDICES_PER_SPRITE) as u32;
                    let index_count = INDICES_PER_SPRITE as u32;

                    render_pass.draw_indexed(index_start..(index_start + index_count), 0, 0..1);
                }
            }

            current_sprite_index += 1;
        }

        // log::info!("Drew {} sprites", draw_count);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_uv_full() {
        let uv = SpriteUV::full();
        assert_eq!(uv.min, Vec2::ZERO);
        assert_eq!(uv.max, Vec2::ONE);
    }

    #[test]
    fn test_sprite_uv_from_sprite_sheet() {
        // 4-frame horizontal sprite sheet
        let uv0 = SpriteUV::from_sprite_sheet(0, 4, false);
        assert_eq!(uv0.min.x, 0.0);
        assert_eq!(uv0.max.x, 0.25);

        let uv1 = SpriteUV::from_sprite_sheet(1, 4, false);
        assert_eq!(uv1.min.x, 0.25);
        assert_eq!(uv1.max.x, 0.5);

        let uv3 = SpriteUV::from_sprite_sheet(3, 4, false);
        assert_eq!(uv3.min.x, 0.75);
        assert_eq!(uv3.max.x, 1.0);
    }

    #[test]
    fn test_sprite_uv_flipped() {
        let uv = SpriteUV::from_sprite_sheet(0, 4, true);
        // When flipped, min.x > max.x
        assert!(uv.min.x > uv.max.x);
    }

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite::new(Vec2::new(10.0, 20.0), Vec2::new(2.0, 3.0));
        assert_eq!(sprite.position, Vec2::new(10.0, 20.0));
        assert_eq!(sprite.size, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_sprite_vertices() {
        let sprite = Sprite::new(Vec2::ZERO, Vec2::ONE);
        let vertices = sprite.vertices();
        assert_eq!(vertices.len(), 4);
    }
}
