// Hot reloading system for assets during development

use super::AssetError;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Tracks file modification times for hot reloading
pub struct HotReloadWatcher {
    /// Map of asset paths to their last modification time
    file_times: HashMap<PathBuf, u64>,

    /// Whether hot reloading is enabled
    enabled: bool,
}

impl HotReloadWatcher {
    /// Create a new hot reload watcher
    pub fn new(enabled: bool) -> Self {
        Self {
            file_times: HashMap::new(),
            enabled,
        }
    }

    /// Register a file for watching
    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let path = path.as_ref().to_path_buf();
        let mtime = Self::get_modification_time(&path)?;
        self.file_times.insert(path, mtime);

        Ok(())
    }

    /// Check if a file has been modified since last check
    pub fn has_changed<P: AsRef<Path>>(&mut self, path: P) -> bool {
        if !self.enabled {
            return false;
        }

        let path = path.as_ref();
        let current_time = match Self::get_modification_time(path) {
            Ok(time) => time,
            Err(_) => return false,
        };

        if let Some(last_time) = self.file_times.get(path) {
            if current_time > *last_time {
                self.file_times.insert(path.to_path_buf(), current_time);
                return true;
            }
        }

        false
    }

    /// Check all watched files for changes
    pub fn check_all(&mut self) -> Vec<PathBuf> {
        if !self.enabled {
            return Vec::new();
        }

        let mut changed = Vec::new();

        // Clone paths to avoid borrow issues
        let paths: Vec<PathBuf> = self.file_times.keys().cloned().collect();

        for path in paths {
            if self.has_changed(&path) {
                changed.push(path);
            }
        }

        changed
    }

    /// Enable or disable hot reloading
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if hot reloading is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all watched files
    pub fn clear(&mut self) {
        self.file_times.clear();
    }

    /// Get the modification time of a file
    fn get_modification_time<P: AsRef<Path>>(path: P) -> Result<u64> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        let duration = mtime
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AssetError::LoadError(format!("Time error: {}", e)))?;
        Ok(duration.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_watcher_disabled() {
        let mut watcher = HotReloadWatcher::new(false);
        assert!(!watcher.is_enabled());

        // Should not detect changes when disabled
        let changed = watcher.check_all();
        assert!(changed.is_empty());
    }

    #[test]
    fn test_watcher_enabled() {
        let watcher = HotReloadWatcher::new(true);
        assert!(watcher.is_enabled());
    }

    #[test]
    fn test_watcher_toggle() {
        let mut watcher = HotReloadWatcher::new(false);
        watcher.set_enabled(true);
        assert!(watcher.is_enabled());

        watcher.set_enabled(false);
        assert!(!watcher.is_enabled());
    }

    #[test]
    fn test_watch_nonexistent_file() {
        let mut watcher = HotReloadWatcher::new(true);
        let result = watcher.watch_file("/nonexistent/file.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_has_changed_nonexistent() {
        let mut watcher = HotReloadWatcher::new(true);
        let changed = watcher.has_changed("/nonexistent/file.png");
        assert!(!changed);
    }

    #[test]
    fn test_watch_real_file() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_watch.txt");

        {
            let mut file = std::fs::File::create(&temp_file).unwrap();
            file.write_all(b"test").unwrap();
        }

        let mut watcher = HotReloadWatcher::new(true);
        let result = watcher.watch_file(&temp_file);
        assert!(result.is_ok());

        // Initially should not have changed
        let changed = watcher.has_changed(&temp_file);
        assert!(!changed);

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_clear() {
        let mut watcher = HotReloadWatcher::new(true);
        assert_eq!(watcher.file_times.len(), 0);

        watcher.clear();
        assert_eq!(watcher.file_times.len(), 0);
    }
}
