use anyhow::Result;
use log::info;
use std::sync::Arc;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

mod core;
mod engine;
mod game;

use engine::game_loop::GameLoop;
use engine::physics::{body::presets, PhysicsWorld, Vector};
use engine::renderer::Renderer;

/// Game world that holds all game state
struct GameWorld {
    renderer: Renderer,
    physics: PhysicsWorld,
    game_loop: GameLoop,

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
        info!("Controls:");
        info!("  D - Toggle debug rendering");
        info!("  R - Reset the falling box");
        info!("  P - Pause/Resume game");

        Ok(Self {
            renderer,
            physics,
            game_loop: GameLoop::new(),
            demo_platform_handle: platform_handle,
            demo_box_handle: box_handle,
        })
    }

    /// Run a single frame: update and render
    fn tick(&mut self) -> Result<()> {
        // Begin frame and get number of updates to run
        let num_updates = self.game_loop.begin_frame();

        // Run fixed timestep updates
        for _ in 0..num_updates {
            self.update();
        }

        // Render the current frame
        self.render()?;

        // Log FPS periodically (every 60 frames)
        if self.game_loop.frame_count() % 60 == 0 {
            info!(
                "FPS: {:.1} | Frame: {} | Updates: {} | Paused: {}",
                self.game_loop.fps(),
                self.game_loop.frame_count(),
                self.game_loop.update_count(),
                self.game_loop.is_paused()
            );
        }

        Ok(())
    }

    /// Update game state with fixed timestep
    /// This should be called multiple times per frame if needed
    fn update(&mut self) {
        let _dt = self.game_loop.fixed_timestep();

        // Step physics simulation with fixed timestep
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
                    KeyCode::KeyP => {
                        game_world.game_loop.toggle_pause();
                    }
                    _ => {}
                },
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Run game loop tick (update + render)
                    if let Err(e) = game_world.tick() {
                        log::error!("Frame error: {:?}", e);
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
