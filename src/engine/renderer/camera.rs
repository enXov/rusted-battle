// Camera and viewport system for 2D rendering

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};

/// 2D camera for sprite rendering
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec2,
    /// Camera zoom level (1.0 = normal, 2.0 = zoomed in 2x)
    pub zoom: f32,
    /// Viewport width
    viewport_width: f32,
    /// Viewport height
    viewport_height: f32,
    /// View-projection matrix
    view_proj: Mat4,
}

impl Camera {
    /// Create a new camera
    pub fn new(position: Vec2, viewport_width: f32, viewport_height: f32) -> Self {
        let mut camera = Self {
            position,
            zoom: 1.0,
            viewport_width,
            viewport_height,
            view_proj: Mat4::IDENTITY,
        };
        camera.update_view_proj();
        camera
    }

    /// Update the view-projection matrix
    fn update_view_proj(&mut self) {
        // Calculate orthographic projection bounds based on zoom
        let half_width = (self.viewport_width / 2.0) / self.zoom;
        let half_height = (self.viewport_height / 2.0) / self.zoom;

        // Create orthographic projection matrix
        let projection = Mat4::orthographic_rh(
            self.position.x - half_width,
            self.position.x + half_width,
            self.position.y - half_height,
            self.position.y + half_height,
            -100.0, // Near plane
            100.0,  // Far plane
        );

        self.view_proj = projection;
    }

    /// Set camera position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.update_view_proj();
    }

    /// Set camera zoom
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.1); // Prevent zoom from being too small
        self.update_view_proj();
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.update_view_proj();
    }

    /// Get the view-projection matrix
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.view_proj
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let normalized_x = (screen_pos.x / self.viewport_width) * 2.0 - 1.0;
        let normalized_y = 1.0 - (screen_pos.y / self.viewport_height) * 2.0;

        let half_width = (self.viewport_width / 2.0) / self.zoom;
        let half_height = (self.viewport_height / 2.0) / self.zoom;

        Vec2::new(
            self.position.x + normalized_x * half_width,
            self.position.y + normalized_y * half_height,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_width = (self.viewport_width / 2.0) / self.zoom;
        let half_height = (self.viewport_height / 2.0) / self.zoom;

        let normalized_x = (world_pos.x - self.position.x) / half_width;
        let normalized_y = (world_pos.y - self.position.y) / half_height;

        Vec2::new(
            (normalized_x + 1.0) * self.viewport_width / 2.0,
            (1.0 - normalized_y) * self.viewport_height / 2.0,
        )
    }

    /// Get the viewport bounds in world coordinates
    pub fn viewport_bounds(&self) -> Viewport {
        let half_width = (self.viewport_width / 2.0) / self.zoom;
        let half_height = (self.viewport_height / 2.0) / self.zoom;

        Viewport {
            min: Vec2::new(self.position.x - half_width, self.position.y - half_height),
            max: Vec2::new(self.position.x + half_width, self.position.y + half_height),
        }
    }
}

/// Viewport bounds in world coordinates
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub min: Vec2,
    pub max: Vec2,
}

impl Viewport {
    /// Check if a point is inside the viewport
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if a rectangle intersects the viewport
    pub fn intersects_rect(&self, center: Vec2, half_size: Vec2) -> bool {
        let rect_min = center - half_size;
        let rect_max = center + half_size;

        rect_max.x >= self.min.x
            && rect_min.x <= self.max.x
            && rect_max.y >= self.min.y
            && rect_min.y <= self.max.y
    }
}

/// Camera uniform for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Create a new camera uniform from a camera
    pub fn new(camera: &Camera) -> Self {
        Self {
            view_proj: camera.view_proj_matrix().to_cols_array_2d(),
        }
    }
}
