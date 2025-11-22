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

use engine::assets::AssetManager;
use engine::game_loop::GameLoop;
use engine::input::{Action, InputManager};
use engine::physics::{body::presets, PhysicsWorld, Vector};
use engine::renderer::Renderer;

/// Game world that holds all game state
struct GameWorld {
    renderer: Renderer,
    physics: PhysicsWorld,
    game_loop: GameLoop,
    input: InputManager,
    assets: AssetManager,

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

        // Initialize input manager (4 players)
        let input = InputManager::new(4);

        // Initialize asset manager
        let asset_path = std::env::current_dir()?.join("assets");
        info!("Asset path: {}", asset_path.display());

        let mut assets = AssetManager::new(asset_path);

        // Example: Create some test textures for prototyping
        assets.create_color_texture(
            renderer.device(),
            renderer.queue(),
            "red",
            [255, 0, 0, 255],
        )?;
        assets.create_color_texture(
            renderer.device(),
            renderer.queue(),
            "green",
            [0, 255, 0, 255],
        )?;
        assets.create_color_texture(
            renderer.device(),
            renderer.queue(),
            "blue",
            [0, 0, 255, 255],
        )?;

        let stats = assets.stats();
        info!(
            "Asset manager initialized with {} textures",
            stats.texture_count
        );

        info!("Physics demo initialized");
        info!("Controls:");
        info!("  P1: WASD to move, 1/2/3 for abilities");
        info!("  P2: IJKL or Arrows to move, U/O/P or Numpad7/8/9 for abilities");
        info!("  F - Toggle debug rendering");
        info!("  R - Reset the falling box");
        info!("  P - Pause/Resume game");
        info!("  ESC - Menu");

        Ok(Self {
            renderer,
            physics,
            game_loop: GameLoop::new(),
            input,
            assets,
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

        // Update input at the end of the frame
        self.input.update();

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

        // Check for hot-reloaded assets (dev mode only)
        #[cfg(debug_assertions)]
        {
            let reloaded = self
                .assets
                .check_hot_reload(self.renderer.device(), self.renderer.queue());
            if !reloaded.is_empty() {
                info!("Hot-reloaded assets: {:?}", reloaded);
            }
        }

        // Process input-driven actions (only when not paused)
        if !self.game_loop.is_paused() {
            self.process_input();
        }

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

    /// Process input and handle game actions
    fn process_input(&mut self) {
        // Handle menu input (global)
        if self.input.any_player_just_pressed(Action::Menu) {
            info!("Menu requested");
            // TODO: Open menu system when implemented
        }

        // Demo: Move the box with P1 input
        if let Some(player1) = self.input.player(0) {
            let (horizontal, _vertical) = player1.get_direction();

            // Set velocity instead of applying impulse (fixes sliding)
            if let Some(body) = self.physics.get_rigid_body_mut(self.demo_box_handle) {
                let mut velocity = *body.linvel();
                velocity.x = horizontal * 10.0; // Set horizontal velocity directly
                body.set_linvel(velocity, true);
            }

            // Jump with W (only if just pressed)
            if player1.just_pressed(Action::Jump) {
                info!("P1 jumped!");
                if let Some(body) = self.physics.get_rigid_body_mut(self.demo_box_handle) {
                    body.apply_impulse(Vector::new(0.0, 30.0), true);
                }
            }

            // Ability demos
            if player1.just_pressed(Action::Ability1) {
                info!("P1 used Ability 1!");
            }
            if player1.just_pressed(Action::Ability2) {
                info!("P1 used Ability 2!");
            }
            if player1.just_pressed(Action::Ability3) {
                info!("P1 used Ability 3!");
            }
        }

        // Demo: Show P2 input
        if let Some(player2) = self.input.player(1) {
            if player2.just_pressed(Action::Ability1) {
                info!("P2 used Ability 1!");
            }
            if player2.just_pressed(Action::Ability2) {
                info!("P2 used Ability 2!");
            }
            if player2.just_pressed(Action::Ability3) {
                info!("P2 used Ability 3!");
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

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        // Handle pause separately (needs to work even when paused)
        if let PhysicalKey::Code(KeyCode::KeyP) = event.physical_key {
            if event.state == winit::event::ElementState::Pressed && !event.repeat {
                self.game_loop.toggle_pause();
                info!(
                    "Game {}",
                    if self.game_loop.is_paused() {
                        "paused"
                    } else {
                        "resumed"
                    }
                );
                return;
            }
        }

        // Process all other input through the input manager
        self.input.process_keyboard_event(event);
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
                            event: ref key_event,
                            ..
                        },
                    ..
                } => {
                    // Let input manager handle all keyboard input
                    game_world.handle_keyboard_input(key_event);

                    // Keep legacy debug keys for now (F and R)
                    if let KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: winit::event::ElementState::Pressed,
                        ..
                    } = key_event
                    {
                        match key_code {
                            KeyCode::KeyF => {
                                game_world.toggle_debug_rendering();
                            }
                            KeyCode::KeyR => {
                                game_world.reset_box();
                            }
                            _ => {}
                        }
                    }
                }
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
