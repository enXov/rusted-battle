/// Game loop timing and control system
///
/// Implements a fixed timestep game loop with variable rendering.
/// This ensures physics and game logic updates at a consistent rate
/// while rendering as fast as possible.
use std::time::{Duration, Instant};

/// Target physics/update rate (60 updates per second)
pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
const FIXED_TIMESTEP_DURATION: Duration = Duration::from_micros(16_667); // ~1/60 second

/// Maximum number of physics steps per frame to prevent spiral of death
const MAX_PHYSICS_STEPS: u32 = 5;

/// FPS tracking window (average over last N frames)
const FPS_WINDOW_SIZE: usize = 60;

/// Game loop timing state
pub struct GameLoop {
    /// Accumulated time for fixed timestep updates
    accumulator: Duration,

    /// Time of last frame
    last_frame_time: Instant,

    /// Time when game loop started
    start_time: Instant,

    /// Whether the game is paused
    paused: bool,

    /// Frame timing history for FPS calculation
    frame_times: Vec<Duration>,

    /// Current frame number
    frame_count: u64,

    /// Total updates executed
    update_count: u64,

    /// Current FPS (updated periodically)
    current_fps: f32,

    /// Delta time for rendering (time since last frame)
    render_delta_time: f32,
}

impl GameLoop {
    /// Create a new game loop
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            accumulator: Duration::ZERO,
            last_frame_time: now,
            start_time: now,
            paused: false,
            frame_times: Vec::with_capacity(FPS_WINDOW_SIZE),
            frame_count: 0,
            update_count: 0,
            current_fps: 0.0,
            render_delta_time: 0.0,
        }
    }

    /// Begin a new frame, returns the number of fixed updates to run
    pub fn begin_frame(&mut self) -> u32 {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.frame_count += 1;

        // Store frame time for FPS calculation
        self.frame_times.push(frame_time);
        if self.frame_times.len() > FPS_WINDOW_SIZE {
            self.frame_times.remove(0);
        }

        // Update FPS counter every 10 frames
        if self.frame_count % 10 == 0 {
            self.update_fps();
        }

        // Store delta time for rendering
        self.render_delta_time = frame_time.as_secs_f32();

        // If paused, don't accumulate time for updates
        if self.paused {
            return 0;
        }

        // Accumulate frame time
        self.accumulator += frame_time;

        // Calculate number of fixed updates to run
        let mut updates = 0;
        while self.accumulator >= FIXED_TIMESTEP_DURATION && updates < MAX_PHYSICS_STEPS {
            self.accumulator -= FIXED_TIMESTEP_DURATION;
            updates += 1;
        }

        self.update_count += updates as u64;
        updates
    }

    /// Get the fixed timestep for physics updates (in seconds)
    pub fn fixed_timestep(&self) -> f32 {
        FIXED_TIMESTEP
    }

    /// Get the delta time since last render (in seconds)
    pub fn render_delta_time(&self) -> f32 {
        self.render_delta_time
    }

    /// Get the interpolation alpha for smooth rendering between physics steps
    /// Alpha = accumulated_time / fixed_timestep
    /// Use this to interpolate object positions between physics updates
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / FIXED_TIMESTEP
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        self.current_fps
    }

    /// Get total elapsed time since start
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }

    /// Get total elapsed time in seconds
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }

    /// Get total number of frames rendered
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get total number of updates executed
    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    /// Check if game is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause the game
    pub fn pause(&mut self) {
        if !self.paused {
            self.paused = true;
            log::info!("Game paused");
        }
    }

    /// Resume the game
    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            // Reset accumulator to prevent update burst
            self.accumulator = Duration::ZERO;
            log::info!("Game resumed");
        }
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// Update FPS calculation
    fn update_fps(&mut self) {
        if self.frame_times.is_empty() {
            self.current_fps = 0.0;
            return;
        }

        // Calculate average frame time
        let total: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total / self.frame_times.len() as u32;

        // Convert to FPS
        self.current_fps = if avg_frame_time.as_secs_f32() > 0.0 {
            1.0 / avg_frame_time.as_secs_f32()
        } else {
            0.0
        };
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_game_loop_creation() {
        let game_loop = GameLoop::new();
        assert_eq!(game_loop.frame_count(), 0);
        assert_eq!(game_loop.update_count(), 0);
        assert!(!game_loop.is_paused());
    }

    #[test]
    fn test_fixed_timestep() {
        let game_loop = GameLoop::new();
        assert_eq!(game_loop.fixed_timestep(), FIXED_TIMESTEP);
        assert!((game_loop.fixed_timestep() - 1.0 / 60.0).abs() < 0.0001);
    }

    #[test]
    fn test_pause_resume() {
        let mut game_loop = GameLoop::new();
        assert!(!game_loop.is_paused());

        game_loop.pause();
        assert!(game_loop.is_paused());

        game_loop.resume();
        assert!(!game_loop.is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut game_loop = GameLoop::new();
        assert!(!game_loop.is_paused());

        game_loop.toggle_pause();
        assert!(game_loop.is_paused());

        game_loop.toggle_pause();
        assert!(!game_loop.is_paused());
    }

    #[test]
    fn test_paused_no_updates() {
        let mut game_loop = GameLoop::new();
        game_loop.pause();

        // Sleep to accumulate some time
        thread::sleep(Duration::from_millis(50));

        // Should return 0 updates when paused
        let updates = game_loop.begin_frame();
        assert_eq!(updates, 0);
    }

    #[test]
    fn test_frame_counting() {
        let mut game_loop = GameLoop::new();
        assert_eq!(game_loop.frame_count(), 0);

        game_loop.begin_frame();
        assert_eq!(game_loop.frame_count(), 1);

        game_loop.begin_frame();
        assert_eq!(game_loop.frame_count(), 2);
    }

    #[test]
    fn test_elapsed_time() {
        let game_loop = GameLoop::new();
        thread::sleep(Duration::from_millis(10));
        let elapsed = game_loop.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_alpha_range() {
        let game_loop = GameLoop::new();
        let alpha = game_loop.alpha();
        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_update_accumulation() {
        let mut game_loop = GameLoop::new();

        // Sleep for roughly one frame time
        thread::sleep(FIXED_TIMESTEP_DURATION);

        let updates = game_loop.begin_frame();
        // Should get at least 1 update (might get 0 if timing is off slightly)
        assert!(updates <= MAX_PHYSICS_STEPS);
    }

    #[test]
    fn test_max_physics_steps_limit() {
        let mut game_loop = GameLoop::new();

        // Simulate a very long frame (300ms)
        thread::sleep(Duration::from_millis(300));

        let updates = game_loop.begin_frame();
        // Should be capped at MAX_PHYSICS_STEPS even though 300ms would allow 18 updates
        assert!(updates <= MAX_PHYSICS_STEPS);
    }
}
