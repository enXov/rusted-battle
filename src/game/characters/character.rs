// Character entity and management

use crate::engine::physics::{
    body::presets, ColliderHandle, PhysicsWorld, RigidBodyHandle, Vector,
};

use super::animation::AnimationPlayer;
use super::state::{CharacterState, CharacterStateMachine};
use super::stats::CharacterStats;

/// Unique identifier for a character
pub type CharacterId = u32;

/// Represents a player-controlled or AI-controlled character in the game
#[derive(Debug)]
pub struct Character {
    /// Unique identifier
    pub id: CharacterId,
    /// Character name (for display)
    pub name: String,
    /// Player index controlling this character (0-3, or None for AI)
    pub player_index: Option<usize>,

    // Physics
    /// Handle to the character's rigid body in the physics world
    pub body_handle: RigidBodyHandle,
    /// Handle to the character's collider
    pub collider_handle: ColliderHandle,

    // Stats and state
    /// Character properties
    pub stats: CharacterStats,
    /// State machine for character behavior
    pub state_machine: CharacterStateMachine,
    /// Animation player for sprite animations
    pub animation: AnimationPlayer,

    // Combat state
    /// Current health
    pub health: i32,
    /// Number of remaining jumps (resets on landing)
    pub jumps_remaining: u8,
    /// Direction character is facing (1 = right, -1 = left)
    pub facing_direction: f32,

    // Input state (updated by input system)
    /// Horizontal movement input (-1 to 1)
    pub input_horizontal: f32,
    /// Whether jump was pressed this frame
    pub input_jump: bool,
    /// Whether duck/down is held
    pub input_duck: bool,
}

impl Character {
    /// Create a new character and add it to the physics world
    pub fn new(
        id: CharacterId,
        name: &str,
        player_index: Option<usize>,
        stats: CharacterStats,
        physics: &mut PhysicsWorld,
        spawn_x: f32,
        spawn_y: f32,
    ) -> Self {
        // Create physics body
        let body = presets::player_body(spawn_x, spawn_y);
        let body_handle = physics.add_rigid_body(body);

        // Create collider with character dimensions
        let collider = presets::player_collider(stats.width, stats.height);
        let collider_handle = physics.add_collider(collider, body_handle);

        Self {
            id,
            name: name.to_string(),
            player_index,
            body_handle,
            collider_handle,
            health: stats.max_health,
            jumps_remaining: stats.max_jumps,
            stats,
            state_machine: CharacterStateMachine::new(),
            animation: AnimationPlayer::with_standard_animations(),
            facing_direction: 1.0,
            input_horizontal: 0.0,
            input_jump: false,
            input_duck: false,
        }
    }

    /// Update character movement based on input
    pub fn update_movement(&mut self, physics: &mut PhysicsWorld, dt: f32) {
        let state = self.state_machine.state();

        // Skip if can't move
        if !state.can_move() {
            return;
        }

        let Some(body) = physics.get_rigid_body_mut(self.body_handle) else {
            return;
        };

        let mut velocity = *body.linvel();
        let is_grounded = self.is_grounded_check(physics);

        // Horizontal movement
        if self.input_horizontal.abs() > 0.1 {
            // Determine movement speed based on ground/air state
            let target_speed = if is_grounded {
                self.input_horizontal * self.stats.move_speed
            } else {
                self.input_horizontal * self.stats.move_speed * self.stats.air_control
            };

            velocity.x = target_speed;

            // Update facing direction
            if self.input_horizontal > 0.0 {
                self.facing_direction = 1.0;
            } else if self.input_horizontal < 0.0 {
                self.facing_direction = -1.0;
            }
        } else if is_grounded {
            // Stop on ground when no input
            velocity.x = 0.0;
        }
        // In air: maintain current velocity when no input (reduced air friction)

        // Apply fast fall
        if self.input_duck && !is_grounded && velocity.y < 0.0 {
            // Only fast fall when moving downward
            let fast_fall_velocity = -self.stats.move_speed * self.stats.fast_fall_multiplier;
            velocity.y = velocity.y.min(fast_fall_velocity);
        }

        // Get mutable body again for the actual update
        if let Some(body) = physics.get_rigid_body_mut(self.body_handle) {
            body.set_linvel(velocity, true);
        }

        // Update state machine
        self.state_machine
            .update(dt, is_grounded, velocity.y, self.input_duck);

        // Update animation based on state
        self.update_animation();
    }

    /// Attempt to jump (checks if jump is allowed)
    pub fn try_jump(&mut self, physics: &mut PhysicsWorld) {
        let state = self.state_machine.state();

        // Can we jump?
        if !state.can_jump() || self.jumps_remaining == 0 {
            return;
        }

        let Some(body) = physics.get_rigid_body_mut(self.body_handle) else {
            return;
        };

        // Apply jump impulse
        let mut velocity = *body.linvel();
        velocity.y = self.stats.jump_force;
        body.set_linvel(velocity, true);

        // Consume a jump
        self.jumps_remaining = self.jumps_remaining.saturating_sub(1);

        // Update state
        self.state_machine.jump();
    }

    /// Check if character is on the ground using raycast
    fn is_grounded_check(&self, physics: &PhysicsWorld) -> bool {
        use rapier2d::prelude::QueryFilter;

        let Some(body) = physics.get_rigid_body(self.body_handle) else {
            return false;
        };

        let position = body.translation();
        let half_height = self.stats.height / 2.0;

        // Cast a ray downward from the character's feet
        let ray_origin = Vector::new(position.x, position.y - half_height + 0.1);
        let ray_direction = Vector::new(0.0, -1.0);
        let max_distance = 0.2;

        physics
            .raycast(
                ray_origin,
                ray_direction,
                max_distance,
                true,
                QueryFilter::default().exclude_rigid_body(self.body_handle),
            )
            .is_some()
    }

    /// Called when character lands on ground
    pub fn on_land(&mut self) {
        // Reset jumps
        self.jumps_remaining = self.stats.max_jumps;

        // Update state machine
        let is_walking = self.input_horizontal.abs() > 0.1;
        self.state_machine.set_grounded(is_walking);
    }

    /// Update animation based on current state
    fn update_animation(&mut self) {
        let state = self.state_machine.state();

        // Play animation for current state
        self.animation.play(state.animation_name());

        // Update flip based on facing direction
        self.animation
            .set_flip_horizontal(self.facing_direction < 0.0);
    }

    /// Update animation timing (called every frame)
    pub fn update_animation_timing(&mut self, dt: f32) {
        self.animation.update(dt);
    }

    /// Get character's current position
    pub fn position(&self, physics: &PhysicsWorld) -> Option<(f32, f32)> {
        physics.get_rigid_body(self.body_handle).map(|body| {
            let pos = body.translation();
            (pos.x, pos.y)
        })
    }

    /// Get character's current velocity
    pub fn velocity(&self, physics: &PhysicsWorld) -> Option<(f32, f32)> {
        physics.get_rigid_body(self.body_handle).map(|body| {
            let vel = body.linvel();
            (vel.x, vel.y)
        })
    }

    /// Set character position (for teleporting/respawning)
    pub fn set_position(&self, physics: &mut PhysicsWorld, x: f32, y: f32) {
        if let Some(body) = physics.get_rigid_body_mut(self.body_handle) {
            body.set_translation(Vector::new(x, y), true);
            body.set_linvel(Vector::new(0.0, 0.0), true);
        }
    }

    /// Apply damage to the character
    pub fn take_damage(&mut self, damage: i32, knockback_x: f32, knockback_y: f32) {
        self.health = (self.health - damage).max(0);

        // Note: Knockback will be applied through physics when combat is implemented
        let _ = (knockback_x, knockback_y); // TODO: Apply knockback via physics

        // Apply hit stun based on damage
        let hit_stun_duration = 0.2 + (damage as f32 * 0.01);
        self.state_machine.apply_hit_stun(hit_stun_duration);

        if self.health <= 0 {
            self.die();
        }
    }

    /// Kill the character
    pub fn die(&mut self) {
        self.state_machine.die();
    }

    /// Respawn the character at a given position
    pub fn respawn(&mut self, physics: &mut PhysicsWorld, x: f32, y: f32) {
        self.set_position(physics, x, y);
        self.health = self.stats.max_health;
        self.jumps_remaining = self.stats.max_jumps;
        self.state_machine.respawn();
    }

    /// Check if character is alive
    pub fn is_alive(&self) -> bool {
        self.state_machine.state() != CharacterState::Dead
    }

    /// Check if character is on ground
    pub fn is_grounded(&self, physics: &PhysicsWorld) -> bool {
        self.is_grounded_check(physics)
    }

    /// Get the current state
    pub fn state(&self) -> CharacterState {
        self.state_machine.state()
    }

    /// Clear input state (called at end of frame)
    pub fn clear_input(&mut self) {
        self.input_jump = false;
        // Note: horizontal and duck inputs are continuous, not cleared
    }
}

/// Manages all characters in the game
#[derive(Debug, Default)]
pub struct CharacterManager {
    characters: Vec<Character>,
    next_id: CharacterId,
}

impl CharacterManager {
    pub fn new() -> Self {
        Self {
            characters: Vec::new(),
            next_id: 0,
        }
    }

    /// Spawn a new character
    pub fn spawn_character(
        &mut self,
        name: &str,
        player_index: Option<usize>,
        stats: CharacterStats,
        physics: &mut PhysicsWorld,
        spawn_x: f32,
        spawn_y: f32,
    ) -> CharacterId {
        let id = self.next_id;
        self.next_id += 1;

        let character = Character::new(id, name, player_index, stats, physics, spawn_x, spawn_y);
        self.characters.push(character);

        id
    }

    /// Get a character by ID
    pub fn get(&self, id: CharacterId) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    /// Get a mutable character by ID
    pub fn get_mut(&mut self, id: CharacterId) -> Option<&mut Character> {
        self.characters.iter_mut().find(|c| c.id == id)
    }

    /// Get a character by player index
    pub fn get_by_player(&self, player_index: usize) -> Option<&Character> {
        self.characters
            .iter()
            .find(|c| c.player_index == Some(player_index))
    }

    /// Get a mutable character by player index
    pub fn get_by_player_mut(&mut self, player_index: usize) -> Option<&mut Character> {
        self.characters
            .iter_mut()
            .find(|c| c.player_index == Some(player_index))
    }

    /// Get all characters
    pub fn all(&self) -> &[Character] {
        &self.characters
    }

    /// Get all characters mutably
    pub fn all_mut(&mut self) -> &mut [Character] {
        &mut self.characters
    }

    /// Update all characters
    pub fn update(&mut self, physics: &mut PhysicsWorld, dt: f32) {
        for character in &mut self.characters {
            // Handle jump input
            if character.input_jump {
                character.try_jump(physics);
            }

            // Update movement
            character.update_movement(physics, dt);

            // Update animation timing
            character.update_animation_timing(dt);

            // Check for landing
            if character.state_machine.state().is_airborne() && character.is_grounded_check(physics)
            {
                character.on_land();
            }

            // Clear per-frame input
            character.clear_input();
        }
    }

    /// Remove a character by ID
    pub fn remove(&mut self, id: CharacterId) -> Option<Character> {
        if let Some(pos) = self.characters.iter().position(|c| c.id == id) {
            Some(self.characters.remove(pos))
        } else {
            None
        }
    }

    /// Get the number of characters
    pub fn count(&self) -> usize {
        self.characters.len()
    }

    /// Get the number of alive characters
    pub fn alive_count(&self) -> usize {
        self.characters.iter().filter(|c| c.is_alive()).count()
    }

    /// Check if a player index is already taken
    pub fn is_player_taken(&self, player_index: usize) -> bool {
        self.characters
            .iter()
            .any(|c| c.player_index == Some(player_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests requiring PhysicsWorld would need a mock or integration tests
    // These tests focus on non-physics functionality

    #[test]
    fn test_character_manager_new() {
        let manager = CharacterManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_character_state_helpers() {
        assert!(CharacterState::Idle.is_grounded());
        assert!(CharacterState::Jumping.is_airborne());
        assert!(CharacterState::Walking.can_move());
        assert!(!CharacterState::Dead.can_move());
    }
}
