// Input configuration and remapping system

use super::action::{Action, InputSource};
use std::collections::HashMap;

/// Input configuration for a single player
/// Maps input sources (keys/buttons) to game actions
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// Player ID this config is for
    player_id: usize,

    /// Mapping from input sources to actions
    bindings: HashMap<InputSource, Action>,

    /// Reverse mapping for quick lookups (action -> all sources)
    action_to_sources: HashMap<Action, Vec<InputSource>>,
}

impl InputConfig {
    /// Create a new input configuration
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            bindings: HashMap::new(),
            action_to_sources: HashMap::new(),
        }
    }

    /// Create a configuration from a list of bindings
    pub fn from_bindings(player_id: usize, bindings: Vec<(InputSource, Action)>) -> Self {
        let mut config = Self::new(player_id);
        for (source, action) in bindings {
            config.bind(source, action);
        }
        config
    }

    /// Get the player ID
    pub fn player_id(&self) -> usize {
        self.player_id
    }

    /// Bind an input source to an action
    pub fn bind(&mut self, source: InputSource, action: Action) {
        // Remove any existing binding for this source
        self.unbind_source(source);

        // Add new binding
        self.bindings.insert(source, action);

        // Update reverse mapping
        self.action_to_sources
            .entry(action)
            .or_insert_with(Vec::new)
            .push(source);
    }

    /// Unbind an input source
    pub fn unbind_source(&mut self, source: InputSource) {
        if let Some(action) = self.bindings.remove(&source) {
            // Update reverse mapping
            if let Some(sources) = self.action_to_sources.get_mut(&action) {
                sources.retain(|s| *s != source);
                if sources.is_empty() {
                    self.action_to_sources.remove(&action);
                }
            }
        }
    }

    /// Unbind all sources for an action
    pub fn unbind_action(&mut self, action: Action) {
        if let Some(sources) = self.action_to_sources.remove(&action) {
            for source in sources {
                self.bindings.remove(&source);
            }
        }
    }

    /// Get the action bound to an input source
    pub fn get_action(&self, source: InputSource) -> Option<Action> {
        self.bindings.get(&source).copied()
    }

    /// Get all input sources bound to an action
    pub fn get_sources(&self, action: Action) -> Vec<InputSource> {
        self.action_to_sources
            .get(&action)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if an input source is bound to any action
    pub fn is_bound(&self, source: InputSource) -> bool {
        self.bindings.contains_key(&source)
    }

    /// Check if an action has any bindings
    pub fn has_binding(&self, action: Action) -> bool {
        self.action_to_sources.contains_key(&action)
    }

    /// Get all bindings as a list
    pub fn get_all_bindings(&self) -> Vec<(InputSource, Action)> {
        self.bindings.iter().map(|(s, a)| (*s, *a)).collect()
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.action_to_sources.clear();
    }

    /// Reset to default bindings for this player
    pub fn reset_to_defaults(&mut self) {
        self.clear();
        let defaults = match self.player_id {
            0 => super::action::default_p1_bindings(),
            _ => Vec::new(), // Players 2+ (remote) have no local bindings by default
        };
        for (source, action) in defaults {
            self.bind(source, action);
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Manager for all player input configurations
#[derive(Debug)]
pub struct InputConfigManager {
    /// Configurations for each player
    configs: Vec<InputConfig>,

    /// Global bindings (not player-specific)
    global_config: InputConfig,
}

impl InputConfigManager {
    /// Create a new config manager with default configurations
    /// Supports up to `max_players` players (typically 4)
    pub fn new(max_players: usize) -> Self {
        let mut configs = Vec::with_capacity(max_players);

        // Set up default configs for each player
        for player_id in 0..max_players {
            let mut config = InputConfig::new(player_id);
            config.reset_to_defaults();
            configs.push(config);
        }

        // Set up global config
        let global_config =
            InputConfig::from_bindings(usize::MAX, super::action::global_bindings());

        Self {
            configs,
            global_config,
        }
    }

    /// Get a player's configuration
    pub fn get_config(&self, player_id: usize) -> Option<&InputConfig> {
        self.configs.get(player_id)
    }

    /// Get a mutable reference to a player's configuration
    pub fn get_config_mut(&mut self, player_id: usize) -> Option<&mut InputConfig> {
        self.configs.get_mut(player_id)
    }

    /// Get the global configuration
    pub fn global_config(&self) -> &InputConfig {
        &self.global_config
    }

    /// Get mutable reference to global configuration
    pub fn global_config_mut(&mut self) -> &mut InputConfig {
        &mut self.global_config
    }

    /// Get the action for a given input source and player
    /// Checks player-specific bindings first, then global bindings
    pub fn get_action(&self, player_id: usize, source: InputSource) -> Option<Action> {
        // Check player-specific bindings first
        if let Some(config) = self.get_config(player_id) {
            if let Some(action) = config.get_action(source) {
                return Some(action);
            }
        }

        // Check global bindings
        self.global_config.get_action(source)
    }

    /// Reset all configurations to defaults
    pub fn reset_all_to_defaults(&mut self) {
        for config in &mut self.configs {
            config.reset_to_defaults();
        }
    }
}

impl Default for InputConfigManager {
    fn default() -> Self {
        Self::new(4) // Default to 4 players
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;

    #[test]
    fn test_config_creation() {
        let config = InputConfig::new(0);
        assert_eq!(config.player_id(), 0);
    }

    #[test]
    fn test_bind_action() {
        let mut config = InputConfig::new(0);
        let source = InputSource::key(KeyCode::KeyA);
        config.bind(source, Action::MoveLeft);

        assert_eq!(config.get_action(source), Some(Action::MoveLeft));
    }

    #[test]
    fn test_unbind_source() {
        let mut config = InputConfig::new(0);
        let source = InputSource::key(KeyCode::KeyA);
        config.bind(source, Action::MoveLeft);
        config.unbind_source(source);

        assert_eq!(config.get_action(source), None);
    }

    #[test]
    fn test_unbind_action() {
        let mut config = InputConfig::new(0);
        let source1 = InputSource::key(KeyCode::KeyA);
        let source2 = InputSource::key(KeyCode::ArrowLeft);

        config.bind(source1, Action::MoveLeft);
        config.bind(source2, Action::MoveLeft);
        config.unbind_action(Action::MoveLeft);

        assert_eq!(config.get_action(source1), None);
        assert_eq!(config.get_action(source2), None);
    }

    #[test]
    fn test_get_sources() {
        let mut config = InputConfig::new(0);
        let source1 = InputSource::key(KeyCode::KeyA);
        let source2 = InputSource::key(KeyCode::ArrowLeft);

        config.bind(source1, Action::MoveLeft);
        config.bind(source2, Action::MoveLeft);

        let sources = config.get_sources(Action::MoveLeft);
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&source1));
        assert!(sources.contains(&source2));
    }

    #[test]
    fn test_rebind_source() {
        let mut config = InputConfig::new(0);
        let source = InputSource::key(KeyCode::KeyA);

        config.bind(source, Action::MoveLeft);
        config.bind(source, Action::MoveRight); // Rebind to different action

        assert_eq!(config.get_action(source), Some(Action::MoveRight));
        assert!(!config.has_binding(Action::MoveLeft));
    }

    #[test]
    fn test_is_bound() {
        let mut config = InputConfig::new(0);
        let source = InputSource::key(KeyCode::KeyA);

        assert!(!config.is_bound(source));
        config.bind(source, Action::MoveLeft);
        assert!(config.is_bound(source));
    }

    #[test]
    fn test_has_binding() {
        let mut config = InputConfig::new(0);
        assert!(!config.has_binding(Action::MoveLeft));

        config.bind(InputSource::key(KeyCode::KeyA), Action::MoveLeft);
        assert!(config.has_binding(Action::MoveLeft));
    }

    #[test]
    fn test_clear() {
        let mut config = InputConfig::new(0);
        config.bind(InputSource::key(KeyCode::KeyA), Action::MoveLeft);
        config.clear();

        assert!(config.get_all_bindings().is_empty());
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut config = InputConfig::new(0);
        config.bind(InputSource::key(KeyCode::KeyZ), Action::MoveLeft);
        config.reset_to_defaults();

        // Should have P1 default bindings
        assert!(config.has_binding(Action::MoveLeft));
        assert!(config.has_binding(Action::Jump));
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = InputConfigManager::new(4);
        assert!(manager.get_config(0).is_some());
        assert!(manager.get_config(3).is_some());
        assert!(manager.get_config(4).is_none());
    }

    #[test]
    fn test_config_manager_get_action() {
        let manager = InputConfigManager::new(4);
        let source = InputSource::key(KeyCode::KeyA); // P1 move left default

        assert_eq!(manager.get_action(0, source), Some(Action::MoveLeft));
    }

    #[test]
    fn test_config_manager_global_bindings() {
        let manager = InputConfigManager::new(4);
        let menu_key = InputSource::key(KeyCode::Escape);

        // Global bindings work for any player
        assert_eq!(manager.get_action(0, menu_key), Some(Action::Menu));
        assert_eq!(manager.get_action(1, menu_key), Some(Action::Menu));
    }

    #[test]
    fn test_config_manager_reset_all() {
        let mut manager = InputConfigManager::new(4);

        // Mess up player 0's config
        if let Some(config) = manager.get_config_mut(0) {
            config.clear();
        }

        manager.reset_all_to_defaults();

        // Should be reset
        assert!(manager.get_config(0).unwrap().has_binding(Action::MoveLeft));
    }

    #[test]
    fn test_from_bindings() {
        let bindings = vec![
            (InputSource::key(KeyCode::KeyA), Action::MoveLeft),
            (InputSource::key(KeyCode::KeyD), Action::MoveRight),
        ];

        let config = InputConfig::from_bindings(0, bindings);
        assert!(config.has_binding(Action::MoveLeft));
        assert!(config.has_binding(Action::MoveRight));
    }
}
