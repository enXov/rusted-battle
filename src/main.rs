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
use engine::physics::{body::presets, PhysicsWorld};
use engine::renderer::{Renderer, Sprite, SpriteUV, TextureHandle};
use game::characters::{CharacterManager, CharacterStats};
use glam::Vec2;

/// Game world that holds all game state
struct GameWorld {
    renderer: Renderer,
    physics: PhysicsWorld,
    game_loop: GameLoop,
    input: InputManager,
    assets: AssetManager,

    // Character system
    characters: CharacterManager,

    // Demo platform (static)
    #[allow(dead_code)]
    demo_platform_handle: engine::physics::RigidBodyHandle,

    // Character sprite texture (for rendering)
    character_texture: Option<TextureHandle>,
}

impl GameWorld {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let mut renderer = Renderer::new(window.clone()).await?;
        let mut physics = PhysicsWorld::new();

        info!("Creating arena platform...");

        // Create a platform (static)
        let platform_body = presets::platform_body(0.0, -5.0);
        let platform_handle = physics.add_rigid_body(platform_body);
        let platform_collider = presets::platform_collider(20.0, 1.0);
        physics.add_collider(platform_collider, platform_handle);

        // Initialize character manager and spawn player 1
        let mut characters = CharacterManager::new();
        let player1_id = characters.spawn_character(
            "Player 1",
            Some(0), // Player index 0
            CharacterStats::standard(),
            &mut physics,
            0.0,  // spawn x
            10.0, // spawn y (above platform)
        );
        info!("Spawned Player 1 character with ID {}", player1_id);

        // Enable physics debug rendering
        renderer.physics_debug_renderer_mut().set_enabled(true);

        // Initialize input manager (4 players)
        let input = InputManager::new(4);

        // Initialize asset manager
        let asset_path = std::env::current_dir()?.join("assets");
        info!("Asset path: {}", asset_path.display());

        let mut assets = AssetManager::new(asset_path);

        // Create test textures for prototyping
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

        // Try to load player sprite sheet into the renderer's texture manager
        let asset_path = std::env::current_dir()?.join("assets/textures/characters/blue_idle.png");
        let character_texture = match renderer.load_texture(&asset_path) {
            Ok(handle) => {
                info!("✓ Loaded player sprite sheet: blue_idle.png");
                Some(handle)
            }
            Err(e) => {
                info!("⚠ Could not load sprite sheet: {}", e);
                info!("   Creating placeholder colored sprite instead");
                // Create a blue placeholder texture
                // Need to borrow device and queue separately to avoid borrow checker issues
                let device = renderer.device() as *const wgpu::Device;
                let queue = renderer.queue() as *const wgpu::Queue;
                let handle = unsafe {
                    renderer.texture_manager_mut().create_color_texture(
                        &*device,
                        &*queue,
                        [50, 100, 255, 255], // Blue color
                        "player_placeholder",
                    )?
                };
                Some(handle)
            }
        };

        let stats = assets.stats();
        info!(
            "Asset manager initialized with {} textures",
            stats.texture_count
        );

        info!("Game initialized with character system");
        info!("Controls:");
        info!("  Player 1: WASD to move, W to jump");
        info!("  Left/Right/Middle mouse buttons for abilities (coming soon)");
        info!("  F - Toggle debug rendering");
        info!("  R - Respawn character");
        info!("  P - Pause/Resume game");
        info!("  ESC - Menu (not implemented yet)");

        Ok(Self {
            renderer,
            physics,
            game_loop: GameLoop::new(),
            input,
            assets,
            characters,
            demo_platform_handle: platform_handle,
            character_texture,
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

        // Log FPS and character state periodically (every 60 frames)
        if self.game_loop.frame_count() % 60 == 0 {
            let character_info = if let Some(character) = self.characters.get_by_player(0) {
                format!(
                    " | P1 State: {:?} | HP: {}",
                    character.state(),
                    character.health
                )
            } else {
                String::new()
            };

            info!(
                "FPS: {:.1} | Frame: {} | Updates: {} | Paused: {}{}",
                self.game_loop.fps(),
                self.game_loop.frame_count(),
                self.game_loop.update_count(),
                self.game_loop.is_paused(),
                character_info
            );
        }

        Ok(())
    }

    /// Update game state with fixed timestep
    fn update(&mut self) {
        let dt = self.game_loop.fixed_timestep();

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

            // Update all characters (movement, physics, animation)
            self.characters.update(&mut self.physics, dt);
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

        // Process Player 1 input
        if let Some(player_input) = self.input.player(0) {
            // Get movement direction
            let (horizontal, _vertical) = player_input.get_direction();

            // Get character and apply input
            if let Some(character) = self.characters.get_by_player_mut(0) {
                // Set movement input
                character.input_horizontal = horizontal;

                // Set duck input
                character.input_duck = player_input.is_held(Action::Duck);

                // Jump (just pressed)
                if player_input.just_pressed(Action::Jump) {
                    character.input_jump = true;
                }

                // Ability demos
                if player_input.just_pressed(Action::Ability1) {
                    info!("P1 used Ability 1! (State: {:?})", character.state());
                }
                if player_input.just_pressed(Action::Ability2) {
                    info!("P1 used Ability 2!");
                }
                if player_input.just_pressed(Action::Ability3) {
                    info!("P1 used Ability 3!");
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        // Add character sprites to the renderer
        for character in self.characters.all() {
            if let Some((x, y)) = character.position(&self.physics) {
                // Get animation frame data
                let frame_data = character.animation.get_frame_data();

                // Calculate UV coordinates for the current animation frame
                // Sprite sheet: 1024x128 pixels, 8 frames (128x128 each) in horizontal row
                // Sprites are at the BOTTOM of each frame

                // Clamp frame to valid range (0-7) for 8 frames
                let frame = frame_data.frame_index.min(7);
                let frame_width = 1.0 / 8.0; // 8 frames = 0.125 each
                let u_min = frame as f32 * frame_width;
                let u_max = u_min + frame_width;

                // Vertical: sprites are at the bottom ~60% of each frame
                let v_min = 0.40; // Start from 40% down (skip empty top)
                let v_max = 1.0; // Go to bottom

                // Sprite faces LEFT in sheet, so:
                // - Character facing RIGHT → flip the sprite
                // - Character facing LEFT → no flip
                let uv = if !frame_data.flip_horizontal {
                    // Character facing right, flip sprite
                    SpriteUV {
                        min: glam::Vec2::new(u_max, v_min),
                        max: glam::Vec2::new(u_min, v_max),
                    }
                } else {
                    // Character facing left, no flip
                    SpriteUV {
                        min: glam::Vec2::new(u_min, v_min),
                        max: glam::Vec2::new(u_max, v_max),
                    }
                };

                // Create sprite for this character
                let mut sprite = Sprite::new(
                    Vec2::new(x, y),
                    Vec2::new(character.stats.width * 2.0, character.stats.height), // Size in world units
                );

                // Set texture if available
                if let Some(texture) = self.character_texture {
                    sprite.texture = Some(texture);
                }
                sprite.uv = uv;
                sprite.z_order = 1.0; // Above background

                self.renderer.add_sprite(sprite);
            }
        }

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

    fn handle_mouse_input(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        // Process mouse input through the input manager
        self.input.process_mouse_button_event(button, state);
    }

    fn respawn_character(&mut self) {
        info!("Respawning Player 1 character");
        if let Some(character) = self.characters.get_by_player_mut(0) {
            character.respawn(&mut self.physics, 0.0, 10.0);
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
    info!("Character System Demo");
    info!("======================");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rusted Battle - Character System")
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
                                game_world.respawn_character();
                            }
                            _ => {}
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    // Let input manager handle mouse input
                    game_world.handle_mouse_input(button, state);
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
