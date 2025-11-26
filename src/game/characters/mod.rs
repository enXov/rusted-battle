// Character system
//
// This module contains everything related to playable characters:
// - Character data structure and management
// - Character stats and properties
// - State machine for character behavior
// - Animation system for sprites

pub mod animation;
pub mod character;
pub mod state;
pub mod stats;

// Re-export commonly used types
pub use animation::{AnimationClip, AnimationFrameData, AnimationPlayer, SpriteSheetConfig};
pub use character::{Character, CharacterId, CharacterManager};
pub use state::{CharacterState, CharacterStateMachine};
pub use stats::CharacterStats;
