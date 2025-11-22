// Input buffering system for reliable input detection

use super::action::Action;
use std::collections::VecDeque;

/// Maximum number of buffered inputs to store
const MAX_BUFFER_SIZE: usize = 30;

/// How long an input remains in the buffer (in frames)
const BUFFER_DURATION: u32 = 5;

/// Represents a single buffered input
#[derive(Debug, Clone, Copy)]
pub struct BufferedInput {
    pub action: Action,
    pub frames_remaining: u32,
}

impl BufferedInput {
    /// Create a new buffered input
    pub fn new(action: Action) -> Self {
        Self {
            action,
            frames_remaining: BUFFER_DURATION,
        }
    }

    /// Decrease the remaining frames
    pub fn age(&mut self) {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
        }
    }

    /// Check if this input has expired
    pub fn is_expired(&self) -> bool {
        self.frames_remaining == 0
    }
}

/// Input buffer for a single player
///
/// Buffers inputs to ensure they're not missed due to timing issues.
/// This is crucial for fighting games where precise input timing matters.
#[derive(Debug)]
pub struct InputBuffer {
    buffer: VecDeque<BufferedInput>,
}

impl InputBuffer {
    /// Create a new input buffer
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_BUFFER_SIZE),
        }
    }

    /// Add an input to the buffer
    pub fn push(&mut self, action: Action) {
        // Don't add duplicate actions if the same action is already buffered
        if !self.buffer.iter().any(|input| input.action == action) {
            self.buffer.push_back(BufferedInput::new(action));

            // Keep buffer size under control
            if self.buffer.len() > MAX_BUFFER_SIZE {
                self.buffer.pop_front();
            }
        }
    }

    /// Check if an action is currently buffered
    pub fn has(&self, action: Action) -> bool {
        self.buffer.iter().any(|input| input.action == action)
    }

    /// Consume an action from the buffer if it exists
    /// Returns true if the action was found and consumed
    pub fn consume(&mut self, action: Action) -> bool {
        if let Some(pos) = self.buffer.iter().position(|input| input.action == action) {
            self.buffer.remove(pos);
            true
        } else {
            false
        }
    }

    /// Update the buffer, aging all inputs and removing expired ones
    /// Call this once per frame
    pub fn update(&mut self) {
        // Age all inputs
        for input in &mut self.buffer {
            input.age();
        }

        // Remove expired inputs
        self.buffer.retain(|input| !input.is_expired());
    }

    /// Clear all buffered inputs
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the number of buffered inputs
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffered_input_creation() {
        let input = BufferedInput::new(Action::Jump);
        assert_eq!(input.action, Action::Jump);
        assert_eq!(input.frames_remaining, BUFFER_DURATION);
    }

    #[test]
    fn test_buffered_input_aging() {
        let mut input = BufferedInput::new(Action::Jump);
        let initial = input.frames_remaining;
        input.age();
        assert_eq!(input.frames_remaining, initial - 1);
    }

    #[test]
    fn test_buffered_input_expiration() {
        let mut input = BufferedInput::new(Action::Jump);
        assert!(!input.is_expired());

        // Age until expired
        for _ in 0..BUFFER_DURATION {
            input.age();
        }
        assert!(input.is_expired());
    }

    #[test]
    fn test_buffer_creation() {
        let buffer = InputBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_push() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        assert_eq!(buffer.len(), 1);
        assert!(buffer.has(Action::Jump));
    }

    #[test]
    fn test_buffer_no_duplicates() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        buffer.push(Action::Jump);
        assert_eq!(buffer.len(), 1, "Buffer should not contain duplicates");
    }

    #[test]
    fn test_buffer_has() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        assert!(buffer.has(Action::Jump));
        assert!(!buffer.has(Action::Ability1));
    }

    #[test]
    fn test_buffer_consume() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        assert!(buffer.consume(Action::Jump));
        assert!(!buffer.has(Action::Jump));
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_consume_nonexistent() {
        let mut buffer = InputBuffer::new();
        assert!(!buffer.consume(Action::Jump));
    }

    #[test]
    fn test_buffer_update() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);

        // Update until the input expires
        for _ in 0..BUFFER_DURATION {
            buffer.update();
        }

        assert!(buffer.is_empty(), "Expired inputs should be removed");
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        buffer.push(Action::Ability2);
        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_multiple_actions() {
        let mut buffer = InputBuffer::new();
        buffer.push(Action::Jump);
        buffer.push(Action::Ability1);
        buffer.push(Action::Duck);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.has(Action::Jump));
        assert!(buffer.has(Action::Ability1));
        assert!(buffer.has(Action::Duck));
    }

    #[test]
    fn test_buffer_max_size() {
        let mut buffer = InputBuffer::new();

        // Push more than MAX_BUFFER_SIZE inputs
        for i in 0..MAX_BUFFER_SIZE + 10 {
            buffer.push(if i % 2 == 0 {
                Action::Jump
            } else {
                Action::Ability1
            });
        }

        assert!(
            buffer.len() <= MAX_BUFFER_SIZE,
            "Buffer should respect max size"
        );
    }
}
