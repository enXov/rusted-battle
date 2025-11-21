use rapier2d::prelude::*;
use std::sync::{Arc, Mutex};

/// Collision groups for filtering what objects can collide with each other
///
/// This is essential for game logic - we need different collision behaviors
/// for players, projectiles, platforms, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionGroups {
    /// Default group - interacts with everything
    Default = 0b0000_0001,

    /// Player characters
    Player = 0b0000_0010,

    /// Projectiles (arrows, bullets, etc.)
    Projectile = 0b0000_0100,

    /// Static platforms and walls
    Platform = 0b0000_1000,

    /// Arena hazards (spikes, lava, etc.)
    Hazard = 0b0001_0000,

    /// Pickups (health, power-ups, etc.)
    Pickup = 0b0010_0000,

    /// Ability effects (explosions, force fields, etc.)
    AbilityEffect = 0b0100_0000,

    /// Sensors (trigger zones, etc.) - don't cause physical collision
    Sensor = 0b1000_0000,
}

impl CollisionGroups {
    /// Convert to rapier2d's InteractionGroups
    pub fn to_interaction_groups(self) -> InteractionGroups {
        let memberships = Group::from_bits_truncate(self as u32);

        // Define what each group can interact with
        let filter = match self {
            // Players collide with platforms, hazards, pickups, and ability effects
            // But not with other players (we'll handle that separately if needed)
            CollisionGroups::Player => Group::from_bits_truncate(
                CollisionGroups::Platform as u32
                    | CollisionGroups::Hazard as u32
                    | CollisionGroups::Pickup as u32
                    | CollisionGroups::AbilityEffect as u32
                    | CollisionGroups::Sensor as u32,
            ),

            // Projectiles collide with players, platforms, and other projectiles
            CollisionGroups::Projectile => Group::from_bits_truncate(
                CollisionGroups::Player as u32
                    | CollisionGroups::Platform as u32
                    | CollisionGroups::Projectile as u32,
            ),

            // Platforms collide with everything except sensors
            CollisionGroups::Platform => Group::from_bits_truncate(
                CollisionGroups::Player as u32
                    | CollisionGroups::Projectile as u32
                    | CollisionGroups::Platform as u32
                    | CollisionGroups::Hazard as u32,
            ),

            // Hazards collide with players only
            CollisionGroups::Hazard => Group::from_bits_truncate(CollisionGroups::Player as u32),

            // Pickups collide with players only
            CollisionGroups::Pickup => Group::from_bits_truncate(CollisionGroups::Player as u32),

            // Ability effects collide with players and projectiles
            CollisionGroups::AbilityEffect => Group::from_bits_truncate(
                CollisionGroups::Player as u32 | CollisionGroups::Projectile as u32,
            ),

            // Sensors interact with everything but don't cause physical collision
            CollisionGroups::Sensor => Group::ALL,

            // Default interacts with everything
            CollisionGroups::Default => Group::ALL,
        };

        InteractionGroups::new(memberships, filter)
    }

    /// Create a sensor version (no physical collision, just detection)
    pub fn as_sensor(self) -> InteractionGroups {
        let groups = self.to_interaction_groups();
        // Keep the same memberships and filters, but we'll set the collider as a sensor
        groups
    }
}

/// Custom collision event for game logic
#[derive(Debug, Clone, Copy)]
pub enum CollisionEvent {
    /// Two colliders started touching
    Started {
        collider1: ColliderHandle,
        collider2: ColliderHandle,
    },

    /// Two colliders stopped touching
    Stopped {
        collider1: ColliderHandle,
        collider2: ColliderHandle,
    },
}

/// Queue for storing collision events during physics step
pub struct CollisionEventQueue {
    events: Arc<Mutex<Vec<CollisionEvent>>>,
}

impl CollisionEventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::with_capacity(32))), // Pre-allocate for common case
        }
    }

    /// Clear all events (call at start of physics step)
    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }

    /// Get all collision events from this frame
    pub fn events(&self) -> Vec<CollisionEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Add a collision event
    fn push(&self, event: CollisionEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

impl Default for CollisionEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

// Implement rapier2d's EventHandler trait for our event queue
impl EventHandler for CollisionEventQueue {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: rapier2d::prelude::CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        match event {
            rapier2d::prelude::CollisionEvent::Started(h1, h2, _flags) => {
                self.push(CollisionEvent::Started {
                    collider1: h1,
                    collider2: h2,
                });
            }
            rapier2d::prelude::CollisionEvent::Stopped(h1, h2, _flags) => {
                self.push(CollisionEvent::Stopped {
                    collider1: h1,
                    collider2: h2,
                });
            }
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &ContactPair,
        _total_force_magnitude: Real,
    ) {
        // We don't need force events for now, but could add them later
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_groups_bits() {
        // Ensure each group has a unique bit
        let groups = [
            CollisionGroups::Default,
            CollisionGroups::Player,
            CollisionGroups::Projectile,
            CollisionGroups::Platform,
            CollisionGroups::Hazard,
            CollisionGroups::Pickup,
            CollisionGroups::AbilityEffect,
            CollisionGroups::Sensor,
        ];

        for (i, group1) in groups.iter().enumerate() {
            for (j, group2) in groups.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        *group1 as u32, *group2 as u32,
                        "Groups must have unique bits"
                    );
                }
            }
        }
    }

    #[test]
    fn test_player_doesnt_collide_with_player() {
        let player_groups = CollisionGroups::Player.to_interaction_groups();
        let player_membership = player_groups.memberships;
        let player_filter = player_groups.filter;

        // Player membership bit should not be in player filter
        assert!(
            !player_filter.contains(player_membership),
            "Players should not collide with other players by default"
        );
    }

    #[test]
    fn test_projectile_collides_with_player() {
        let projectile_groups = CollisionGroups::Projectile.to_interaction_groups();
        let player_bit = Group::from_bits_truncate(CollisionGroups::Player as u32);

        assert!(
            projectile_groups.filter.contains(player_bit),
            "Projectiles should collide with players"
        );
    }
}
