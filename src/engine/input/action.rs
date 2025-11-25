// Game action definitions and mappings

use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Represents all possible in-game actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Movement
    MoveLeft,
    MoveRight,
    Jump,
    Duck,

    // Abilities (3 slots - mouse buttons)
    Ability1, // Left mouse
    Ability2, // Right mouse
    Ability3, // Middle mouse

    // Meta actions
    Pause,
    Menu,
}

/// Represents an input source (keyboard key, mouse button, or controller button)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    // Future: Add controller support
    // GamepadButton(gilrs::Button),
}

impl InputSource {
    /// Create a keyboard input source
    pub fn key(code: KeyCode) -> Self {
        Self::Keyboard(code)
    }

    /// Create a mouse button input source
    pub fn mouse(button: MouseButton) -> Self {
        Self::Mouse(button)
    }
}

/// Default keyboard/mouse bindings for Player 1
pub fn default_p1_bindings() -> Vec<(InputSource, Action)> {
    vec![
        // Movement (WASD - standard gaming layout)
        (InputSource::key(KeyCode::KeyA), Action::MoveLeft),
        (InputSource::key(KeyCode::KeyD), Action::MoveRight),
        (InputSource::key(KeyCode::KeyW), Action::Jump),
        (InputSource::key(KeyCode::KeyS), Action::Duck),
        // Abilities (mouse buttons)
        (InputSource::mouse(MouseButton::Left), Action::Ability1),
        (InputSource::mouse(MouseButton::Right), Action::Ability2),
        (InputSource::mouse(MouseButton::Middle), Action::Ability3),
    ]
}

/// Global bindings (not player-specific)
pub fn global_bindings() -> Vec<(InputSource, Action)> {
    vec![
        (InputSource::key(KeyCode::Escape), Action::Menu),
        // Note: Pause (P) is handled separately in main.rs to work when game is paused
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Jump, Action::Jump);
        assert_ne!(Action::Jump, Action::Duck);
    }

    #[test]
    fn test_input_source_keyboard_creation() {
        let source = InputSource::key(KeyCode::KeyA);
        assert_eq!(source, InputSource::Keyboard(KeyCode::KeyA));
    }

    #[test]
    fn test_input_source_mouse_creation() {
        let source = InputSource::mouse(MouseButton::Left);
        assert_eq!(source, InputSource::Mouse(MouseButton::Left));
    }

    #[test]
    fn test_default_p1_bindings_exist() {
        let bindings = default_p1_bindings();
        assert!(!bindings.is_empty());
        assert!(bindings.len() >= 7); // At least movement + abilities
    }

    #[test]
    fn test_default_p1_bindings_use_mouse_for_abilities() {
        let bindings = default_p1_bindings();

        // Find ability bindings
        let ability1_binding = bindings
            .iter()
            .find(|(_, action)| *action == Action::Ability1);
        let ability2_binding = bindings
            .iter()
            .find(|(_, action)| *action == Action::Ability2);
        let ability3_binding = bindings
            .iter()
            .find(|(_, action)| *action == Action::Ability3);

        // Verify they use mouse buttons
        assert!(matches!(
            ability1_binding,
            Some((InputSource::Mouse(MouseButton::Left), _))
        ));
        assert!(matches!(
            ability2_binding,
            Some((InputSource::Mouse(MouseButton::Right), _))
        ));
        assert!(matches!(
            ability3_binding,
            Some((InputSource::Mouse(MouseButton::Middle), _))
        ));
    }

    #[test]
    fn test_global_bindings_exist() {
        let bindings = global_bindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_no_duplicate_inputs_in_p1() {
        let bindings = default_p1_bindings();
        let mut seen_sources = std::collections::HashSet::new();
        for (source, _) in bindings {
            assert!(
                seen_sources.insert(source),
                "Duplicate input source found in P1 bindings"
            );
        }
    }
}
