// Character animation system

use std::collections::HashMap;

/// A single animation clip
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Name of the animation (e.g., "idle", "walk", "jump")
    pub name: String,
    /// Number of frames in the animation
    pub frame_count: usize,
    /// Duration of each frame in seconds
    pub frame_duration: f32,
    /// Whether the animation loops
    pub looping: bool,
    /// Frame index where the loop restarts (if looping)
    pub loop_start: usize,
}

impl AnimationClip {
    /// Create a new animation clip
    pub fn new(name: &str, frame_count: usize, fps: f32, looping: bool) -> Self {
        Self {
            name: name.to_string(),
            frame_count,
            frame_duration: 1.0 / fps,
            looping,
            loop_start: 0,
        }
    }

    /// Create a looping animation
    pub fn looping(name: &str, frame_count: usize, fps: f32) -> Self {
        Self::new(name, frame_count, fps, true)
    }

    /// Create a one-shot animation (plays once)
    pub fn one_shot(name: &str, frame_count: usize, fps: f32) -> Self {
        Self::new(name, frame_count, fps, false)
    }

    /// Set the frame where looping restarts
    pub fn with_loop_start(mut self, frame: usize) -> Self {
        self.loop_start = frame.min(self.frame_count.saturating_sub(1));
        self
    }

    /// Get the total duration of one animation cycle
    pub fn total_duration(&self) -> f32 {
        self.frame_count as f32 * self.frame_duration
    }
}

/// Manages animation playback for a character
#[derive(Debug)]
pub struct AnimationPlayer {
    /// All available animations
    animations: HashMap<String, AnimationClip>,
    /// Currently playing animation name
    current_animation: String,
    /// Current frame index
    current_frame: usize,
    /// Time elapsed in current frame
    frame_timer: f32,
    /// Whether the animation is playing
    playing: bool,
    /// Playback speed multiplier (1.0 = normal)
    playback_speed: f32,
    /// Whether the sprite should be flipped horizontally
    flip_horizontal: bool,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            current_animation: String::new(),
            current_frame: 0,
            frame_timer: 0.0,
            playing: true,
            playback_speed: 1.0,
            flip_horizontal: false,
        }
    }

    /// Create an animation player with standard character animations
    pub fn with_standard_animations() -> Self {
        let mut player = Self::new();

        // Add standard character animations
        // Idle: 8 frames at 10 FPS (matches new sprite sheet)
        player.add_animation(AnimationClip::looping("idle", 8, 10.0));
        // Other animations use idle for now until we have more sprite sheets
        player.add_animation(AnimationClip::looping("walk", 8, 12.0));
        player.add_animation(AnimationClip::looping("jump", 8, 10.0));
        player.add_animation(AnimationClip::looping("fall", 8, 10.0));
        player.add_animation(AnimationClip::looping("fast_fall", 8, 12.0));
        player.add_animation(AnimationClip::looping("duck", 8, 10.0));
        player.add_animation(AnimationClip::looping("hit", 8, 12.0));
        player.add_animation(AnimationClip::looping("dead", 8, 10.0));

        // Start with idle
        player.play("idle");

        player
    }

    /// Add an animation clip
    pub fn add_animation(&mut self, clip: AnimationClip) {
        self.animations.insert(clip.name.clone(), clip);
    }

    /// Play an animation by name
    pub fn play(&mut self, name: &str) {
        if self.current_animation != name {
            self.current_animation = name.to_string();
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.playing = true;
        }
    }

    /// Play an animation from the beginning, even if it's the same
    pub fn play_from_start(&mut self, name: &str) {
        self.current_animation = name.to_string();
        self.current_frame = 0;
        self.frame_timer = 0.0;
        self.playing = true;
    }

    /// Pause the current animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resume the current animation
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Stop and reset the current animation
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_frame = 0;
        self.frame_timer = 0.0;
    }

    /// Set playback speed (1.0 = normal, 2.0 = double speed)
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.0);
    }

    /// Set horizontal flip state
    pub fn set_flip_horizontal(&mut self, flip: bool) {
        self.flip_horizontal = flip;
    }

    /// Get horizontal flip state
    pub fn is_flipped_horizontal(&self) -> bool {
        self.flip_horizontal
    }

    /// Update the animation (called every frame)
    pub fn update(&mut self, dt: f32) {
        if !self.playing {
            return;
        }

        let Some(clip) = self.animations.get(&self.current_animation) else {
            return;
        };

        self.frame_timer += dt * self.playback_speed;

        while self.frame_timer >= clip.frame_duration {
            self.frame_timer -= clip.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= clip.frame_count {
                if clip.looping {
                    self.current_frame = clip.loop_start;
                } else {
                    // Stay on last frame
                    self.current_frame = clip.frame_count - 1;
                    self.playing = false;
                }
            }
        }
    }

    /// Get the current animation name
    pub fn current_animation(&self) -> &str {
        &self.current_animation
    }

    /// Get the current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Check if the animation is playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Check if the current animation has finished (for non-looping animations)
    pub fn is_finished(&self) -> bool {
        if let Some(clip) = self.animations.get(&self.current_animation) {
            !clip.looping && self.current_frame >= clip.frame_count - 1 && !self.playing
        } else {
            true
        }
    }

    /// Get animation data for rendering (frame index and flip state)
    pub fn get_frame_data(&self) -> AnimationFrameData {
        // Get the clip's frame count for bounds checking
        let max_frame = self
            .animations
            .get(&self.current_animation)
            .map(|clip| clip.frame_count.saturating_sub(1))
            .unwrap_or(0);

        // Clamp frame index to valid range
        let safe_frame = self.current_frame.min(max_frame);

        AnimationFrameData {
            animation_name: self.current_animation.clone(),
            frame_index: safe_frame,
            flip_horizontal: self.flip_horizontal,
        }
    }

    /// Get the clip info for the current animation
    pub fn current_clip(&self) -> Option<&AnimationClip> {
        self.animations.get(&self.current_animation)
    }
}

/// Data needed to render the current animation frame
#[derive(Debug, Clone)]
pub struct AnimationFrameData {
    pub animation_name: String,
    pub frame_index: usize,
    pub flip_horizontal: bool,
}

/// Sprite sheet configuration for a character
#[derive(Debug, Clone)]
pub struct SpriteSheetConfig {
    /// Width of each frame in pixels
    pub frame_width: u32,
    /// Height of each frame in pixels
    pub frame_height: u32,
    /// Number of columns in the sprite sheet
    pub columns: u32,
    /// Mapping from animation name to row index in sprite sheet
    pub animation_rows: HashMap<String, u32>,
}

impl SpriteSheetConfig {
    /// Create a new sprite sheet configuration
    pub fn new(frame_width: u32, frame_height: u32, columns: u32) -> Self {
        Self {
            frame_width,
            frame_height,
            columns,
            animation_rows: HashMap::new(),
        }
    }

    /// Add an animation row mapping
    pub fn with_animation(mut self, name: &str, row: u32) -> Self {
        self.animation_rows.insert(name.to_string(), row);
        self
    }

    /// Create a standard character sprite sheet config (64x64 frames, 8 columns)
    pub fn standard_character() -> Self {
        Self::new(64, 64, 8)
            .with_animation("idle", 0)
            .with_animation("walk", 1)
            .with_animation("jump", 2)
            .with_animation("fall", 3)
            .with_animation("fast_fall", 3) // Same row as fall
            .with_animation("duck", 4)
            .with_animation("hit", 5)
            .with_animation("dead", 6)
    }

    /// Calculate UV coordinates for a given animation frame
    /// Returns (u_min, v_min, u_max, v_max) in normalized coordinates [0, 1]
    pub fn get_frame_uvs(
        &self,
        animation_name: &str,
        frame_index: usize,
        texture_width: u32,
        texture_height: u32,
    ) -> (f32, f32, f32, f32) {
        let row = *self.animation_rows.get(animation_name).unwrap_or(&0);
        let col = frame_index as u32 % self.columns;

        let u_min = (col * self.frame_width) as f32 / texture_width as f32;
        let v_min = (row * self.frame_height) as f32 / texture_height as f32;
        let u_max = ((col + 1) * self.frame_width) as f32 / texture_width as f32;
        let v_max = ((row + 1) * self.frame_height) as f32 / texture_height as f32;

        (u_min, v_min, u_max, v_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_clip_creation() {
        let clip = AnimationClip::looping("idle", 4, 8.0);
        assert_eq!(clip.name, "idle");
        assert_eq!(clip.frame_count, 4);
        assert_eq!(clip.frame_duration, 0.125); // 1/8
        assert!(clip.looping);
    }

    #[test]
    fn test_animation_clip_duration() {
        let clip = AnimationClip::looping("walk", 6, 10.0);
        assert_eq!(clip.total_duration(), 0.6); // 6 frames * 0.1s
    }

    #[test]
    fn test_animation_player_play() {
        let mut player = AnimationPlayer::with_standard_animations();
        assert_eq!(player.current_animation(), "idle");

        player.play("walk");
        assert_eq!(player.current_animation(), "walk");
        assert_eq!(player.current_frame(), 0);
    }

    #[test]
    fn test_animation_player_update() {
        let mut player = AnimationPlayer::new();
        player.add_animation(AnimationClip::looping("test", 4, 10.0)); // 0.1s per frame
        player.play("test");

        player.update(0.15); // 1.5 frames worth
        assert_eq!(player.current_frame(), 1);

        player.update(0.1);
        assert_eq!(player.current_frame(), 2);
    }

    #[test]
    fn test_animation_looping() {
        let mut player = AnimationPlayer::new();
        player.add_animation(AnimationClip::looping("test", 3, 10.0));
        player.play("test");

        // Advance through all frames
        player.update(0.35); // 3.5 frames
        assert_eq!(player.current_frame(), 0); // Should loop back
        assert!(player.is_playing());
    }

    #[test]
    fn test_animation_one_shot() {
        let mut player = AnimationPlayer::new();
        player.add_animation(AnimationClip::one_shot("test", 3, 10.0));
        player.play("test");

        // Advance past all frames
        player.update(0.5);
        assert_eq!(player.current_frame(), 2); // Last frame
        assert!(!player.is_playing());
        assert!(player.is_finished());
    }

    #[test]
    fn test_sprite_sheet_uvs() {
        let config = SpriteSheetConfig::new(64, 64, 8).with_animation("idle", 0);

        // First frame of idle
        let (u_min, v_min, u_max, v_max) = config.get_frame_uvs("idle", 0, 512, 512);
        assert_eq!(u_min, 0.0);
        assert_eq!(v_min, 0.0);
        assert_eq!(u_max, 0.125); // 64/512
        assert_eq!(v_max, 0.125);

        // Second frame
        let (u_min, _, u_max, _) = config.get_frame_uvs("idle", 1, 512, 512);
        assert_eq!(u_min, 0.125);
        assert_eq!(u_max, 0.25);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut player = AnimationPlayer::with_standard_animations();
        assert!(!player.is_flipped_horizontal());

        player.set_flip_horizontal(true);
        assert!(player.is_flipped_horizontal());

        let frame_data = player.get_frame_data();
        assert!(frame_data.flip_horizontal);
    }

    #[test]
    fn test_playback_speed() {
        let mut player = AnimationPlayer::new();
        player.add_animation(AnimationClip::looping("test", 4, 10.0));
        player.play("test");
        player.set_playback_speed(2.0);

        player.update(0.1); // Should advance 2 frames at 2x speed
        assert_eq!(player.current_frame(), 2);
    }
}
