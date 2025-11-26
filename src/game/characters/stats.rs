// Character stats - ALL PLAYERS HAVE THE SAME STATS
// Differentiation comes from SKILLS/ABILITIES, not base stats

/// Fixed character stats - same for all players
/// This ensures fair gameplay where skill choice matters, not stat differences
#[derive(Debug, Clone)]
pub struct CharacterStats {
    // Movement
    /// Maximum horizontal movement speed (units/second)
    pub move_speed: f32,
    /// Jump impulse strength
    pub jump_force: f32,
    /// Number of jumps allowed
    pub max_jumps: u8,
    /// Air control multiplier (0.0 = no air control, 1.0 = full control)
    pub air_control: f32,

    // Physics
    /// Gravity multiplier
    pub gravity_scale: f32,
    /// Fall speed multiplier when holding down
    pub fast_fall_multiplier: f32,

    // Combat
    /// Base health points
    pub max_health: i32,

    // Dimensions (for physics collider)
    /// Character width in world units
    pub width: f32,
    /// Character height in world units
    pub height: f32,
}

/// The ONE character stats used by all players
/// Balanced for fair competitive play
pub const BASE_STATS: CharacterStats = CharacterStats {
    // Movement - responsive but not too fast
    move_speed: 10.0,
    jump_force: 30.0,
    max_jumps: 1,
    air_control: 0.8,

    // Physics
    gravity_scale: 1.0,
    fast_fall_multiplier: 2.0,

    // Combat
    max_health: 100,

    // Dimensions - fits the blob sprite nicely
    width: 1.0,
    height: 2.0,
};

impl Default for CharacterStats {
    fn default() -> Self {
        BASE_STATS
    }
}

impl CharacterStats {
    /// Get the standard character stats (same for all players)
    pub fn standard() -> Self {
        BASE_STATS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stats() {
        let stats = CharacterStats::default();
        assert_eq!(stats.move_speed, 10.0);
        assert_eq!(stats.max_jumps, 1);
        assert_eq!(stats.max_health, 100);
    }

    #[test]
    fn test_standard_equals_default() {
        let standard = CharacterStats::standard();
        let default = CharacterStats::default();
        assert_eq!(standard.move_speed, default.move_speed);
        assert_eq!(standard.max_health, default.max_health);
    }

    #[test]
    fn test_all_players_same_stats() {
        // This test documents that all players use identical stats
        let player1 = CharacterStats::standard();
        let player2 = CharacterStats::standard();
        let player3 = CharacterStats::standard();

        assert_eq!(player1.move_speed, player2.move_speed);
        assert_eq!(player2.max_health, player3.max_health);
    }
}
