// Input handling system
//
// This module provides a complete input system for handling keyboard and controller input
// across multiple players, with support for input buffering, remapping, and configuration.
//
// ## Architecture
//
// - `action`: Defines game actions and default key bindings
// - `buffer`: Input buffering for reliable input detection
// - `player`: Per-player input state management
// - `config`: Input configuration and remapping system
// - `manager`: Main input manager coordinating everything
//
// ## Usage Example
//
// ```rust
// use engine::input::{InputManager, Action};
//
// // Create input manager for 4 players
// let mut input_manager = InputManager::new(4);
//
// // In your event loop, process keyboard events
// input_manager.process_keyboard_event(&key_event);
//
// // At the end of each frame, update the input state
// input_manager.update();
//
// // Query input state
// if let Some(player) = input_manager.player(0) {
//     if player.just_pressed(Action::Jump) {
//         // Player 0 just pressed jump!
//     }
//
//     let (horizontal, vertical) = player.get_direction();
//     // Use direction for movement
// }
// ```

pub mod action;
pub mod buffer;
pub mod config;
pub mod manager;
pub mod player;

// Re-export commonly used types
pub use action::{Action, InputSource};
pub use config::{InputConfig, InputConfigManager};
pub use manager::InputManager;
pub use player::PlayerInput;
