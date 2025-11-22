// Input manager - Main coordination system for all input

use super::action::{Action, InputSource};
use super::config::InputConfigManager;
use super::player::PlayerInput;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::PhysicalKey;

/// Main input manager that coordinates all input for all players
pub struct InputManager {
    /// Configuration manager for all players
    config: InputConfigManager,

    /// Input state for each player
    players: Vec<PlayerInput>,

    /// Maximum number of supported players
    max_players: usize,
}

impl InputManager {
    /// Create a new input manager
    pub fn new(max_players: usize) -> Self {
        let config = InputConfigManager::new(max_players);
        let mut players = Vec::with_capacity(max_players);

        for player_id in 0..max_players {
            players.push(PlayerInput::new(player_id));
        }

        Self {
            config,
            players,
            max_players,
        }
    }

    /// Process a keyboard event from winit
    pub fn process_keyboard_event(&mut self, event: &KeyEvent) {
        // Only process physical key presses
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let source = InputSource::key(key_code);

            // Check each player's bindings
            for player_id in 0..self.max_players {
                if let Some(action) = self.config.get_action(player_id, source) {
                    if let Some(player) = self.players.get_mut(player_id) {
                        match event.state {
                            ElementState::Pressed => {
                                if !event.repeat {
                                    // Only register if not a key repeat
                                    player.press(action);
                                }
                            }
                            ElementState::Released => {
                                player.release(action);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Update all player input states for a new frame
    /// Call this once per frame after processing all events
    pub fn update(&mut self) {
        for player in &mut self.players {
            player.update();
        }
    }

    /// Get input state for a specific player
    pub fn player(&self, player_id: usize) -> Option<&PlayerInput> {
        self.players.get(player_id)
    }

    /// Get mutable input state for a specific player
    pub fn player_mut(&mut self, player_id: usize) -> Option<&mut PlayerInput> {
        self.players.get_mut(player_id)
    }

    /// Get the configuration manager
    pub fn config(&self) -> &InputConfigManager {
        &self.config
    }

    /// Get mutable configuration manager
    pub fn config_mut(&mut self) -> &mut InputConfigManager {
        &mut self.config
    }

    /// Check if any player pressed a specific action this frame
    pub fn any_player_just_pressed(&self, action: Action) -> bool {
        self.players.iter().any(|p| p.just_pressed(action))
    }

    /// Check if any player is pressing a specific action
    pub fn any_player_pressed(&self, action: Action) -> bool {
        self.players.iter().any(|p| p.is_pressed(action))
    }

    /// Get a list of all players who just pressed an action
    pub fn get_players_who_pressed(&self, action: Action) -> Vec<usize> {
        self.players
            .iter()
            .filter(|p| p.just_pressed(action))
            .map(|p| p.player_id())
            .collect()
    }

    /// Reset all player input states
    pub fn reset_all(&mut self) {
        for player in &mut self.players {
            player.reset();
        }
    }

    /// Get the number of players
    pub fn num_players(&self) -> usize {
        self.max_players
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new(4) // Default to 4 players
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = InputManager::new(4);
        assert_eq!(manager.num_players(), 4);
        assert!(manager.player(0).is_some());
        assert!(manager.player(3).is_some());
        assert!(manager.player(4).is_none());
    }

    #[test]
    fn test_direct_input_manipulation() {
        let mut manager = InputManager::new(4);

        // Directly manipulate player input for testing
        if let Some(player) = manager.player_mut(0) {
            player.press(Action::MoveLeft);
        }

        assert!(manager.player(0).unwrap().is_pressed(Action::MoveLeft));
    }

    #[test]
    fn test_input_release() {
        let mut manager = InputManager::new(4);

        // Press and then release
        if let Some(player) = manager.player_mut(0) {
            player.press(Action::Jump);
        }
        manager.update();

        if let Some(player) = manager.player_mut(0) {
            player.release(Action::Jump);
        }

        assert!(!manager.player(0).unwrap().is_pressed(Action::Jump));
        assert!(manager.player(0).unwrap().just_released(Action::Jump));
    }

    #[test]
    fn test_multiple_players() {
        let mut manager = InputManager::new(4);

        // P1 presses move left
        if let Some(player) = manager.player_mut(0) {
            player.press(Action::MoveLeft);
        }

        // P2 presses move left
        if let Some(player) = manager.player_mut(1) {
            player.press(Action::MoveLeft);
        }

        assert!(manager.player(0).unwrap().is_pressed(Action::MoveLeft));
        assert!(manager.player(1).unwrap().is_pressed(Action::MoveLeft));
    }

    #[test]
    fn test_update_clears_just_pressed() {
        let mut manager = InputManager::new(4);

        if let Some(player) = manager.player_mut(0) {
            player.press(Action::Ability1);
        }
        assert!(manager.player(0).unwrap().just_pressed(Action::Ability1));

        manager.update();
        assert!(!manager.player(0).unwrap().just_pressed(Action::Ability1));
        assert!(manager.player(0).unwrap().is_pressed(Action::Ability1));
    }

    #[test]
    fn test_any_player_pressed() {
        let mut manager = InputManager::new(4);

        if let Some(player) = manager.player_mut(2) {
            player.press(Action::Duck);
        }

        assert!(manager.any_player_pressed(Action::Duck));
        assert!(!manager.any_player_pressed(Action::Jump));
    }

    #[test]
    fn test_any_player_just_pressed() {
        let mut manager = InputManager::new(4);

        if let Some(player) = manager.player_mut(1) {
            player.press(Action::Ability2);
        }

        assert!(manager.any_player_just_pressed(Action::Ability2));

        manager.update();
        assert!(!manager.any_player_just_pressed(Action::Ability2));
    }

    #[test]
    fn test_get_players_who_pressed() {
        let mut manager = InputManager::new(4);

        // P1 presses jump
        if let Some(player) = manager.player_mut(0) {
            player.press(Action::Jump);
        }

        // P2 presses jump
        if let Some(player) = manager.player_mut(1) {
            player.press(Action::Jump);
        }

        let players = manager.get_players_who_pressed(Action::Jump);
        assert_eq!(players.len(), 2);
        assert!(players.contains(&0));
        assert!(players.contains(&1));
    }

    #[test]
    fn test_reset_all() {
        let mut manager = InputManager::new(4);

        if let Some(player) = manager.player_mut(0) {
            player.press(Action::Ability3);
        }
        assert!(manager.player(0).unwrap().is_pressed(Action::Ability3));

        manager.reset_all();
        assert!(!manager.player(0).unwrap().is_pressed(Action::Ability3));
    }

    #[test]
    fn test_config_access() {
        let manager = InputManager::new(4);
        let config = manager.config();

        // Check that P1 has default bindings
        assert!(config.get_config(0).is_some());
        assert!(config.get_config(1).is_some());
    }

    #[test]
    fn test_multiple_actions_per_player() {
        let mut manager = InputManager::new(4);

        if let Some(player) = manager.player_mut(0) {
            player.press(Action::Jump);
            player.press(Action::Ability1);
            player.press(Action::MoveRight);
        }

        let player = manager.player(0).unwrap();
        assert!(player.is_pressed(Action::Jump));
        assert!(player.is_pressed(Action::Ability1));
        assert!(player.is_pressed(Action::MoveRight));
    }
}
