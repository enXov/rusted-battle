// Per-player input state management

use super::action::Action;
use super::buffer::InputBuffer;
use std::collections::HashSet;

/// Represents the input state for a single player
#[derive(Debug)]
pub struct PlayerInput {
    /// Player ID (0-3 for up to 4 players)
    player_id: usize,

    /// Actions that are currently pressed this frame
    pressed: HashSet<Action>,

    /// Actions that were just pressed this frame (press events)
    just_pressed: HashSet<Action>,

    /// Actions that were just released this frame (release events)
    just_released: HashSet<Action>,

    /// Actions that were pressed in the previous frame
    previous_pressed: HashSet<Action>,

    /// Input buffer for delayed/buffered inputs
    buffer: InputBuffer,
}

impl PlayerInput {
    /// Create a new player input state
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            previous_pressed: HashSet::new(),
            buffer: InputBuffer::new(),
        }
    }

    /// Get the player ID
    pub fn player_id(&self) -> usize {
        self.player_id
    }

    /// Check if an action is currently pressed
    pub fn is_pressed(&self, action: Action) -> bool {
        self.pressed.contains(&action)
    }

    /// Check if an action was just pressed this frame
    pub fn just_pressed(&self, action: Action) -> bool {
        self.just_pressed.contains(&action)
    }

    /// Check if an action was just released this frame
    pub fn just_released(&self, action: Action) -> bool {
        self.just_released.contains(&action)
    }

    /// Check if an action is held (pressed for multiple frames)
    pub fn is_held(&self, action: Action) -> bool {
        self.pressed.contains(&action) && self.previous_pressed.contains(&action)
    }

    /// Check if an action is buffered
    pub fn is_buffered(&self, action: Action) -> bool {
        self.buffer.has(action)
    }

    /// Consume a buffered action
    /// Returns true if the action was buffered and consumed
    pub fn consume_buffered(&mut self, action: Action) -> bool {
        self.buffer.consume(action)
    }

    /// Register an action press
    pub(crate) fn press(&mut self, action: Action) {
        if !self.pressed.contains(&action) {
            self.just_pressed.insert(action);
            self.pressed.insert(action);
            // Also add to buffer for reliable input detection
            self.buffer.push(action);
        }
    }

    /// Register an action release
    pub(crate) fn release(&mut self, action: Action) {
        if self.pressed.contains(&action) {
            self.just_released.insert(action);
            self.pressed.remove(&action);
        }
    }

    /// Update input state for a new frame
    /// Call this once per frame after processing all events
    pub(crate) fn update(&mut self) {
        // Clear frame-specific state
        self.just_pressed.clear();
        self.just_released.clear();

        // Save current pressed state for next frame
        self.previous_pressed = self.pressed.clone();

        // Update buffer
        self.buffer.update();
    }

    /// Reset all input state
    pub fn reset(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
        self.previous_pressed.clear();
        self.buffer.clear();
    }

    /// Get all currently pressed actions
    pub fn get_pressed_actions(&self) -> Vec<Action> {
        self.pressed.iter().copied().collect()
    }

    /// Get all actions that were just pressed
    pub fn get_just_pressed_actions(&self) -> Vec<Action> {
        self.just_pressed.iter().copied().collect()
    }

    /// Get directional input as a normalized vector (-1.0 to 1.0)
    /// Returns (horizontal, vertical)
    pub fn get_direction(&self) -> (f32, f32) {
        let mut horizontal = 0.0;
        let mut vertical = 0.0;

        if self.is_pressed(Action::MoveLeft) {
            horizontal -= 1.0;
        }
        if self.is_pressed(Action::MoveRight) {
            horizontal += 1.0;
        }
        if self.is_pressed(Action::Duck) {
            vertical -= 1.0;
        }
        if self.is_pressed(Action::Jump) {
            vertical += 1.0;
        }

        (horizontal, vertical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_input_creation() {
        let input = PlayerInput::new(0);
        assert_eq!(input.player_id(), 0);
        assert!(!input.is_pressed(Action::Jump));
    }

    #[test]
    fn test_press_action() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        assert!(input.is_pressed(Action::Jump));
        assert!(input.just_pressed(Action::Jump));
    }

    #[test]
    fn test_release_action() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        input.update();
        input.release(Action::Jump);
        assert!(!input.is_pressed(Action::Jump));
        assert!(input.just_released(Action::Jump));
    }

    #[test]
    fn test_just_pressed_cleared_on_update() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        assert!(input.just_pressed(Action::Jump));

        input.update();
        assert!(input.is_pressed(Action::Jump));
        assert!(!input.just_pressed(Action::Jump));
    }

    #[test]
    fn test_held_detection() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        assert!(!input.is_held(Action::Jump)); // Not held on first frame

        input.update();
        assert!(input.is_held(Action::Jump)); // Held after update
    }

    #[test]
    fn test_buffered_input() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        assert!(input.is_buffered(Action::Jump));
    }

    #[test]
    fn test_consume_buffered() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        input.update();
        input.release(Action::Jump);

        assert!(input.consume_buffered(Action::Jump));
        assert!(!input.is_buffered(Action::Jump));
    }

    #[test]
    fn test_reset() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        input.press(Action::Ability1);
        input.reset();

        assert!(!input.is_pressed(Action::Jump));
        assert!(!input.is_pressed(Action::Ability1));
        assert!(input.get_pressed_actions().is_empty());
    }

    #[test]
    fn test_get_pressed_actions() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        input.press(Action::Ability1);

        let actions = input.get_pressed_actions();
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&Action::Jump));
        assert!(actions.contains(&Action::Ability1));
    }

    #[test]
    fn test_get_direction_neutral() {
        let input = PlayerInput::new(0);
        let (h, v) = input.get_direction();
        assert_eq!(h, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_get_direction_horizontal() {
        let mut input = PlayerInput::new(0);
        input.press(Action::MoveRight);
        let (h, _v) = input.get_direction();
        assert_eq!(h, 1.0);

        input.release(Action::MoveRight);
        input.press(Action::MoveLeft);
        let (h, _v) = input.get_direction();
        assert_eq!(h, -1.0);
    }

    #[test]
    fn test_get_direction_vertical() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        let (_h, v) = input.get_direction();
        assert_eq!(v, 1.0);

        input.release(Action::Jump);
        input.press(Action::Duck);
        let (_h, v) = input.get_direction();
        assert_eq!(v, -1.0);
    }

    #[test]
    fn test_multiple_presses_same_action() {
        let mut input = PlayerInput::new(0);
        input.press(Action::Jump);
        input.press(Action::Jump); // Press again

        let actions = input.get_pressed_actions();
        assert_eq!(actions.len(), 1, "Should not duplicate actions");
    }

    #[test]
    fn test_release_unpressed_action() {
        let mut input = PlayerInput::new(0);
        input.release(Action::Jump); // Release without pressing

        assert!(!input.just_released(Action::Jump));
    }
}
