use rapier2d::prelude::*;
use wgpu::util::DeviceExt;

/// Debug renderer for physics objects
/// Renders colliders, rigid bodies, and other physics debug info
pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertices: Vec<DebugVertex>,
    indices: Vec<u16>,
    enabled: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugUniforms {
    view_proj: [[f32; 4]; 4],
}

impl DebugRenderer {
    /// Create a new debug renderer
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        view_proj_matrix: [[f32; 4]; 4],
    ) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Physics Debug Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/debug.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Physics Debug Uniform Buffer"),
            contents: bytemuck::cast_slice(&[DebugUniforms {
                view_proj: view_proj_matrix,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Physics Debug Bind Group Layout"),
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

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Physics Debug Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Physics Debug Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Physics Debug Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<DebugVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // position
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // color
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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

        // Create initial buffers (will be resized as needed)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Debug Vertex Buffer"),
            size: 1024, // Initial size
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Debug Index Buffer"),
            size: 1024, // Initial size
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            vertices: Vec::new(),
            indices: Vec::new(),
            enabled: false, // Disabled by default
        }
    }

    /// Enable or disable debug rendering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if debug rendering is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update the view-projection matrix
    pub fn update_view_proj(&self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4]) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[DebugUniforms { view_proj }]),
        );
    }

    /// Prepare debug geometry for rendering
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rigid_bodies: &RigidBodySet,
        colliders: &ColliderSet,
    ) {
        if !self.enabled {
            return;
        }

        self.vertices.clear();
        self.indices.clear();

        // Render all colliders
        let identity = Isometry::identity();
        for (_handle, collider) in colliders.iter() {
            let parent_body = collider.parent().and_then(|h| rigid_bodies.get(h));
            let body_pos = parent_body.map(|b| b.position()).unwrap_or(&identity);

            // Get color based on body type
            let color = if let Some(body) = parent_body {
                match body.body_type() {
                    RigidBodyType::Dynamic => [0.0, 1.0, 0.0, 0.8], // Green for dynamic
                    RigidBodyType::Fixed => [0.5, 0.5, 0.5, 0.8],   // Gray for static
                    RigidBodyType::KinematicPositionBased => [0.0, 0.5, 1.0, 0.8], // Blue for kinematic
                    RigidBodyType::KinematicVelocityBased => [0.0, 0.5, 1.0, 0.8],
                }
            } else {
                [1.0, 1.0, 1.0, 0.8] // White for no parent
            };

            // Draw collider shape
            self.draw_collider_shape(collider, body_pos, color);
        }

        // Update buffers if we have geometry
        if !self.vertices.is_empty() {
            let vertex_size = (self.vertices.len() * std::mem::size_of::<DebugVertex>()) as u64;
            let index_size = (self.indices.len() * std::mem::size_of::<u16>()) as u64;

            // Resize buffers if needed
            if vertex_size > self.vertex_buffer.size() {
                self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Physics Debug Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
            } else {
                queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
            }

            if index_size > self.index_buffer.size() {
                self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Physics Debug Index Buffer"),
                    contents: bytemuck::cast_slice(&self.indices),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                });
            } else {
                queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
            }
        }
    }

    /// Draw a collider shape
    fn draw_collider_shape(
        &mut self,
        collider: &Collider,
        transform: &Isometry<Real>,
        color: [f32; 4],
    ) {
        match collider.shape().shape_type() {
            ShapeType::Ball => {
                if let Some(ball) = collider.shape().as_ball() {
                    self.draw_circle(transform, ball.radius, color);
                }
            }
            ShapeType::Cuboid => {
                if let Some(cuboid) = collider.shape().as_cuboid() {
                    self.draw_box(transform, cuboid.half_extents, color);
                }
            }
            ShapeType::Capsule => {
                if let Some(capsule) = collider.shape().as_capsule() {
                    self.draw_capsule(transform, capsule, color);
                }
            }
            _ => {
                // For other shapes, draw a simple cross
                self.draw_cross(transform, 0.5, color);
            }
        }
    }

    /// Draw a circle
    fn draw_circle(&mut self, transform: &Isometry<Real>, radius: Real, color: [f32; 4]) {
        const SEGMENTS: usize = 16;
        let start_idx = self.vertices.len() as u16;

        for i in 0..SEGMENTS {
            let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius as f32;
            let y = angle.sin() * radius as f32;

            // Transform point
            let point = transform * point![x as Real, y as Real];

            self.vertices.push(DebugVertex {
                position: [point.x as f32, point.y as f32],
                color,
            });

            // Add line segment
            let next = (i + 1) % SEGMENTS;
            self.indices.push(start_idx + i as u16);
            self.indices.push(start_idx + next as u16);
        }
    }

    /// Draw a box
    fn draw_box(
        &mut self,
        transform: &Isometry<Real>,
        half_extents: Vector<Real>,
        color: [f32; 4],
    ) {
        let start_idx = self.vertices.len() as u16;

        // Four corners
        let corners = [
            [-half_extents.x, -half_extents.y],
            [half_extents.x, -half_extents.y],
            [half_extents.x, half_extents.y],
            [-half_extents.x, half_extents.y],
        ];

        for corner in &corners {
            let point = transform * point![corner[0], corner[1]];
            self.vertices.push(DebugVertex {
                position: [point.x as f32, point.y as f32],
                color,
            });
        }

        // Four edges
        for i in 0..4 {
            self.indices.push(start_idx + i);
            self.indices.push(start_idx + (i + 1) % 4);
        }
    }

    /// Draw a capsule
    fn draw_capsule(&mut self, transform: &Isometry<Real>, capsule: &Capsule, color: [f32; 4]) {
        // Draw the two circles at the ends
        let a_transform = transform * Translation::from(capsule.segment.a.coords);
        let b_transform = transform * Translation::from(capsule.segment.b.coords);

        self.draw_circle(&a_transform, capsule.radius, color);
        self.draw_circle(&b_transform, capsule.radius, color);

        // Draw lines connecting the circles
        let dir = capsule.segment.b - capsule.segment.a;
        let len = dir.norm();
        if len > 0.0 {
            let dir = dir / len;
            let perp = vector![-dir.y, dir.x] * capsule.radius;

            let points = [
                capsule.segment.a + perp,
                capsule.segment.b + perp,
                capsule.segment.b - perp,
                capsule.segment.a - perp,
            ];

            let start_idx = self.vertices.len() as u16;
            for point in &points {
                let world_point = transform * point;
                self.vertices.push(DebugVertex {
                    position: [world_point.x as f32, world_point.y as f32],
                    color,
                });
            }

            // Two side lines
            self.indices.push(start_idx);
            self.indices.push(start_idx + 1);
            self.indices.push(start_idx + 2);
            self.indices.push(start_idx + 3);
        }
    }

    /// Draw a cross (for unsupported shapes)
    fn draw_cross(&mut self, transform: &Isometry<Real>, size: Real, color: [f32; 4]) {
        let start_idx = self.vertices.len() as u16;

        let points = [
            point![-size, 0.0],
            point![size, 0.0],
            point![0.0, -size],
            point![0.0, size],
        ];

        for point in &points {
            let world_point = transform * point;
            self.vertices.push(DebugVertex {
                position: [world_point.x as f32, world_point.y as f32],
                color,
            });
        }

        self.indices.push(start_idx);
        self.indices.push(start_idx + 1);
        self.indices.push(start_idx + 2);
        self.indices.push(start_idx + 3);
    }

    /// Render the debug geometry
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.enabled || self.indices.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}
