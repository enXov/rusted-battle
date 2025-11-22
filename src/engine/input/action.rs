// Game action definitions and mappings

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

/// Represents an input source (keyboard key or controller button)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Keyboard(KeyCode),
    // Future: Add controller support
    // GamepadButton(gilrs::Button),
}

impl InputSource {
    /// Create a keyboard input source
    pub fn key(code: KeyCode) -> Self {
        Self::Keyboard(code)
    }
}

/// Default keyboard bindings for Player 1
pub fn default_p1_bindings() -> Vec<(InputSource, Action)> {
    vec![
        // Movement (WASD - standard gaming layout)
        (InputSource::key(KeyCode::KeyA), Action::MoveLeft),
        (InputSource::key(KeyCode::KeyD), Action::MoveRight),
        (InputSource::key(KeyCode::KeyW), Action::Jump),
        (InputSource::key(KeyCode::KeyS), Action::Duck),
        // Abilities (will be mouse buttons in actual game, using keys for now)
        (InputSource::key(KeyCode::Digit1), Action::Ability1),
        (InputSource::key(KeyCode::Digit2), Action::Ability2),
        (InputSource::key(KeyCode::Digit3), Action::Ability3),
    ]
}

/// Default keyboard bindings for Player 2
pub fn default_p2_bindings() -> Vec<(InputSource, Action)> {
    vec![
        // Movement (Arrow keys + IJKL as alternative)
        (InputSource::key(KeyCode::ArrowLeft), Action::MoveLeft),
        (InputSource::key(KeyCode::KeyJ), Action::MoveLeft),
        (InputSource::key(KeyCode::ArrowRight), Action::MoveRight),
        (InputSource::key(KeyCode::KeyL), Action::MoveRight),
        (InputSource::key(KeyCode::ArrowUp), Action::Jump),
        (InputSource::key(KeyCode::KeyI), Action::Jump),
        (InputSource::key(KeyCode::ArrowDown), Action::Duck),
        (InputSource::key(KeyCode::KeyK), Action::Duck),
        // Abilities (Numpad for those who have it)
        (InputSource::key(KeyCode::Numpad7), Action::Ability1),
        (InputSource::key(KeyCode::Numpad8), Action::Ability2),
        (InputSource::key(KeyCode::Numpad9), Action::Ability3),
        // Alternative abilities for keyboards without numpad
        (InputSource::key(KeyCode::KeyU), Action::Ability1),
        (InputSource::key(KeyCode::KeyO), Action::Ability2),
        (InputSource::key(KeyCode::KeyP), Action::Ability3),
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
    fn test_input_source_creation() {
        let source = InputSource::key(KeyCode::KeyA);
        assert_eq!(source, InputSource::Keyboard(KeyCode::KeyA));
    }

    #[test]
    fn test_default_p1_bindings_exist() {
        let bindings = default_p1_bindings();
        assert!(!bindings.is_empty());
        assert!(bindings.len() >= 7); // At least movement + abilities
    }

    #[test]
    fn test_default_p2_bindings_exist() {
        let bindings = default_p2_bindings();
        assert!(!bindings.is_empty());
        assert!(bindings.len() >= 7);
    }

    #[test]
    fn test_global_bindings_exist() {
        let bindings = global_bindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_no_duplicate_keys_in_p1() {
        let bindings = default_p1_bindings();
        let mut seen_keys = std::collections::HashSet::new();
        for (source, _) in bindings {
            assert!(
                seen_keys.insert(source),
                "Duplicate key found in P1 bindings"
            );
        }
    }

    #[test]
    fn test_no_duplicate_keys_in_p2() {
        let bindings = default_p2_bindings();
        let mut seen_keys = std::collections::HashSet::new();
        for (source, _) in bindings {
            assert!(
                seen_keys.insert(source),
                "Duplicate key found in P2 bindings"
            );
        }
    }
}
