use anyhow::Result;
use log::info;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

mod core;
mod engine;
mod game;

use engine::physics::{body::presets, PhysicsWorld, Vector};
use engine::renderer::Renderer;

/// Game world that holds all game state
struct GameWorld {
    renderer: Renderer,
    physics: PhysicsWorld,
    last_update: Instant,

    // Demo objects
    #[allow(dead_code)]
    demo_platform_handle: engine::physics::RigidBodyHandle,
    demo_box_handle: engine::physics::RigidBodyHandle,
}

impl GameWorld {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let mut renderer = Renderer::new(window.clone()).await?;
        let mut physics = PhysicsWorld::new();

        info!("Creating demo physics objects...");

        // Create a platform (static)
        let platform_body = presets::platform_body(0.0, -5.0);
        let platform_handle = physics.add_rigid_body(platform_body);
        let platform_collider = presets::platform_collider(20.0, 1.0);
        physics.add_collider(platform_collider, platform_handle);

        // Create a falling box (dynamic)
        let box_body = presets::player_body(0.0, 10.0);
        let box_handle = physics.add_rigid_body(box_body);
        let box_collider = presets::player_collider(1.0, 2.0);
        physics.add_collider(box_collider, box_handle);

        // Enable physics debug rendering
        renderer.physics_debug_renderer_mut().set_enabled(true);

        info!("Physics demo initialized");
        info!("Press D to toggle debug rendering");
        info!("Press R to reset the falling box");

        Ok(Self {
            renderer,
            physics,
            last_update: Instant::now(),
            demo_platform_handle: platform_handle,
            demo_box_handle: box_handle,
        })
    }

    fn update(&mut self) {
        let now = Instant::now();
        let _dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Step physics simulation
        self.physics.step();

        // Check for collision events
        for event in self.physics.get_collision_events() {
            match event {
                engine::physics::CollisionEvent::Started {
                    collider1,
                    collider2,
                } => {
                    info!("Collision started: {:?} <-> {:?}", collider1, collider2);
                }
                engine::physics::CollisionEvent::Stopped {
                    collider1,
                    collider2,
                } => {
                    info!("Collision stopped: {:?} <-> {:?}", collider1, collider2);
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        // Prepare physics debug rendering
        let debug_data = self.physics.debug_data();
        let device = self.renderer.device() as *const wgpu::Device;
        let queue = self.renderer.queue() as *const wgpu::Queue;

        // SAFETY: We need to work around Rust's borrow checker here.
        // The prepare() call doesn't actually need mutable access to the renderer's
        // device and queue, just read access. This is safe because we're not
        // actually mutating them.
        unsafe {
            self.renderer.physics_debug_renderer_mut().prepare(
                &*device,
                &*queue,
                debug_data.rigid_bodies,
                debug_data.colliders,
            );
        }

        // Render the frame
        self.renderer.render()?;

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    fn toggle_debug_rendering(&mut self) {
        let debug_renderer = self.renderer.physics_debug_renderer_mut();
        let enabled = debug_renderer.is_enabled();
        debug_renderer.set_enabled(!enabled);
        info!(
            "Physics debug rendering: {}",
            if !enabled { "ON" } else { "OFF" }
        );
    }

    fn reset_box(&mut self) {
        info!("Resetting falling box");
        if let Some(body) = self.physics.get_rigid_body_mut(self.demo_box_handle) {
            body.set_translation(Vector::new(0.0, 10.0), true);
            body.set_linvel(Vector::new(0.0, 0.0), true);
            body.set_angvel(0.0, true);
        }
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Rusted Battle...");
    info!("======================");
    info!("Physics Integration Demo");
    info!("======================");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rusted Battle - Physics Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_resizable(true)
            .build(&event_loop)?,
    );

    info!("Window created successfully");

    // Initialize game world
    let mut game_world = pollster::block_on(GameWorld::new(window.clone()))?;
    info!("Game world initialized successfully");

    // Main event loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Close requested, shutting down...");
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size),
                    ..
                } => {
                    info!("Window resized to {:?}", physical_size);
                    game_world.resize(physical_size);
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(key_code),
                                    state: winit::event::ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                } => match key_code {
                    KeyCode::KeyD => {
                        game_world.toggle_debug_rendering();
                    }
                    KeyCode::KeyR => {
                        game_world.reset_box();
                    }
                    _ => {}
                },
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Update game state
                    game_world.update();

                    // Render frame
                    if let Err(e) = game_world.render() {
                        log::error!("Render error: {:?}", e);
                    }
                }
                Event::AboutToWait => {
                    // Request redraw on next frame
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;

    Ok(())
}
