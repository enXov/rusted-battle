// Character state machine

/// Represents the current state of a character
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
    /// Standing still on ground
    Idle,
    /// Moving horizontally on ground
    Walking,
    /// In the air, moving upward
    Jumping,
    /// In the air, moving downward
    Falling,
    /// Fast falling (holding down while in air)
    FastFalling,
    /// Crouching/ducking on ground
    Ducking,
    /// Taking damage/hit stun
    HitStun,
    /// Character is dead/KO'd
    Dead,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self::Idle
    }
}

impl CharacterState {
    /// Check if the character is on the ground
    pub fn is_grounded(&self) -> bool {
        matches!(self, Self::Idle | Self::Walking | Self::Ducking)
    }

    /// Check if the character is in the air
    pub fn is_airborne(&self) -> bool {
        matches!(self, Self::Jumping | Self::Falling | Self::FastFalling)
    }

    /// Check if the character can move horizontally
    pub fn can_move(&self) -> bool {
        !matches!(self, Self::HitStun | Self::Dead)
    }

    /// Check if the character can jump
    pub fn can_jump(&self) -> bool {
        !matches!(self, Self::HitStun | Self::Dead | Self::Ducking)
    }

    /// Check if the character can use abilities
    pub fn can_use_ability(&self) -> bool {
        !matches!(self, Self::HitStun | Self::Dead)
    }

    /// Check if the character can duck
    pub fn can_duck(&self) -> bool {
        self.is_grounded() && !matches!(self, Self::Ducking)
    }

    /// Get the animation name for this state
    pub fn animation_name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walking => "walk",
            Self::Jumping => "jump",
            Self::Falling => "fall",
            Self::FastFalling => "fast_fall",
            Self::Ducking => "duck",
            Self::HitStun => "hit",
            Self::Dead => "dead",
        }
    }
}

/// State machine that handles character state transitions
#[derive(Debug)]
pub struct CharacterStateMachine {
    current_state: CharacterState,
    previous_state: CharacterState,
    state_time: f32,
    hit_stun_remaining: f32,
}

impl Default for CharacterStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: CharacterState::Idle,
            previous_state: CharacterState::Idle,
            state_time: 0.0,
            hit_stun_remaining: 0.0,
        }
    }

    /// Get the current state
    pub fn state(&self) -> CharacterState {
        self.current_state
    }

    /// Get the previous state
    pub fn previous_state(&self) -> CharacterState {
        self.previous_state
    }

    /// Get time spent in current state
    pub fn state_time(&self) -> f32 {
        self.state_time
    }

    /// Check if state just changed this frame
    pub fn state_just_changed(&self) -> bool {
        self.state_time == 0.0
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: CharacterState) {
        if self.current_state != new_state {
            self.previous_state = self.current_state;
            self.current_state = new_state;
            self.state_time = 0.0;
        }
    }

    /// Force transition even to the same state (resets state time)
    pub fn force_transition(&mut self, new_state: CharacterState) {
        self.previous_state = self.current_state;
        self.current_state = new_state;
        self.state_time = 0.0;
    }

    /// Update the state machine (called every frame)
    pub fn update(&mut self, dt: f32, is_grounded: bool, velocity_y: f32, is_ducking: bool) {
        self.state_time += dt;

        // Handle hit stun timer
        if self.current_state == CharacterState::HitStun {
            self.hit_stun_remaining -= dt;
            if self.hit_stun_remaining <= 0.0 {
                self.hit_stun_remaining = 0.0;
                // Exit hit stun
                self.transition(if is_grounded {
                    CharacterState::Idle
                } else {
                    CharacterState::Falling
                });
            }
            return;
        }

        // Don't update if dead
        if self.current_state == CharacterState::Dead {
            return;
        }

        // State transitions based on physics
        if is_grounded {
            // On ground
            if is_ducking && self.current_state.can_duck() {
                self.transition(CharacterState::Ducking);
            } else if !is_ducking && self.current_state == CharacterState::Ducking {
                self.transition(CharacterState::Idle);
            }
            // Airborne to grounded transitions are handled by set_grounded()
        } else {
            // In air
            match self.current_state {
                CharacterState::Jumping if velocity_y <= 0.0 => {
                    self.transition(CharacterState::Falling);
                }
                CharacterState::Falling if is_ducking => {
                    self.transition(CharacterState::FastFalling);
                }
                CharacterState::FastFalling if !is_ducking => {
                    self.transition(CharacterState::Falling);
                }
                CharacterState::Idle | CharacterState::Walking | CharacterState::Ducking => {
                    // Walked off ledge
                    self.transition(CharacterState::Falling);
                }
                _ => {}
            }
        }
    }

    /// Called when character starts walking
    pub fn start_walking(&mut self) {
        if self.current_state == CharacterState::Idle {
            self.transition(CharacterState::Walking);
        }
    }

    /// Called when character stops walking
    pub fn stop_walking(&mut self) {
        if self.current_state == CharacterState::Walking {
            self.transition(CharacterState::Idle);
        }
    }

    /// Called when character jumps
    pub fn jump(&mut self) {
        if self.current_state.can_jump() {
            self.transition(CharacterState::Jumping);
        }
    }

    /// Called when character lands on ground
    pub fn set_grounded(&mut self, is_walking: bool) {
        if self.current_state.is_airborne() {
            self.transition(if is_walking {
                CharacterState::Walking
            } else {
                CharacterState::Idle
            });
        }
    }

    /// Apply hit stun to the character
    pub fn apply_hit_stun(&mut self, duration: f32) {
        if self.current_state != CharacterState::Dead {
            self.transition(CharacterState::HitStun);
            self.hit_stun_remaining = duration;
        }
    }

    /// Kill the character
    pub fn die(&mut self) {
        self.transition(CharacterState::Dead);
    }

    /// Respawn the character
    pub fn respawn(&mut self) {
        self.transition(CharacterState::Falling);
        self.hit_stun_remaining = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = CharacterStateMachine::new();
        assert_eq!(sm.state(), CharacterState::Idle);
    }

    #[test]
    fn test_state_transition() {
        let mut sm = CharacterStateMachine::new();
        sm.transition(CharacterState::Walking);
        assert_eq!(sm.state(), CharacterState::Walking);
        assert_eq!(sm.previous_state(), CharacterState::Idle);
    }

    #[test]
    fn test_state_just_changed() {
        let mut sm = CharacterStateMachine::new();
        sm.transition(CharacterState::Walking);
        assert!(sm.state_just_changed());
        sm.update(0.1, true, 0.0, false);
        assert!(!sm.state_just_changed());
    }

    #[test]
    fn test_jump_transition() {
        let mut sm = CharacterStateMachine::new();
        sm.jump();
        assert_eq!(sm.state(), CharacterState::Jumping);
    }

    #[test]
    fn test_jumping_to_falling() {
        let mut sm = CharacterStateMachine::new();
        sm.jump();
        // Simulate being in air with downward velocity
        sm.update(0.1, false, -1.0, false);
        assert_eq!(sm.state(), CharacterState::Falling);
    }

    #[test]
    fn test_falling_to_grounded() {
        let mut sm = CharacterStateMachine::new();
        sm.transition(CharacterState::Falling);
        sm.set_grounded(false);
        assert_eq!(sm.state(), CharacterState::Idle);
    }

    #[test]
    fn test_grounded_states() {
        assert!(CharacterState::Idle.is_grounded());
        assert!(CharacterState::Walking.is_grounded());
        assert!(CharacterState::Ducking.is_grounded());
        assert!(!CharacterState::Jumping.is_grounded());
        assert!(!CharacterState::Falling.is_grounded());
    }

    #[test]
    fn test_hit_stun() {
        let mut sm = CharacterStateMachine::new();
        sm.apply_hit_stun(0.5);
        assert_eq!(sm.state(), CharacterState::HitStun);
        assert!(!sm.state().can_move());

        // Simulate hit stun duration
        sm.update(0.6, true, 0.0, false);
        assert_ne!(sm.state(), CharacterState::HitStun);
    }

    #[test]
    fn test_death() {
        let mut sm = CharacterStateMachine::new();
        sm.die();
        assert_eq!(sm.state(), CharacterState::Dead);
        assert!(!sm.state().can_move());
        assert!(!sm.state().can_jump());
    }

    #[test]
    fn test_respawn() {
        let mut sm = CharacterStateMachine::new();
        sm.die();
        sm.respawn();
        assert_eq!(sm.state(), CharacterState::Falling);
    }

    #[test]
    fn test_animation_names() {
        assert_eq!(CharacterState::Idle.animation_name(), "idle");
        assert_eq!(CharacterState::Walking.animation_name(), "walk");
        assert_eq!(CharacterState::Jumping.animation_name(), "jump");
    }

    #[test]
    fn test_fast_falling() {
        let mut sm = CharacterStateMachine::new();
        sm.transition(CharacterState::Falling);
        sm.update(0.1, false, -5.0, true); // Holding duck while falling
        assert_eq!(sm.state(), CharacterState::FastFalling);
    }
}
