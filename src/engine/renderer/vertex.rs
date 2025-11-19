// Vertex structure for 2D sprite rendering

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};

/// Vertex for 2D sprite rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Position in 3D space (z for layering)
    pub position: [f32; 3],
    /// Texture coordinates (UV)
    pub tex_coords: [f32; 2],
    /// Vertex color (RGBA)
    pub color: [f32; 4],
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: Vec3, tex_coords: Vec2, color: Vec4) -> Self {
        Self {
            position: position.to_array(),
            tex_coords: tex_coords.to_array(),
            color: color.to_array(),
        }
    }

    /// Get the vertex buffer layout descriptor
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Tex Coords
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
