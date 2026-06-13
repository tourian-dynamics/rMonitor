//! Status bar with automatic decay timer for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).

use std::time::{Duration, Instant};

const DEFAULT_DECAY: Duration = Duration::from_secs(4);

/// Tracks a status message that automatically reverts to a default after `decay`.
#[derive(Debug, Clone)]
pub struct StatusBar {
    pub message: String,
    pub last_set: Option<Instant>,
    pub default_message: String,
    pub decay: Duration,
}

impl StatusBar {
    /// Create a new StatusBar whose "default" message is shown after `decay`.
    pub fn new(default_message: impl Into<String>) -> Self {
        Self {
            message: String::new(),
            last_set: None,
            default_message: default_message.into(),
            decay: DEFAULT_DECAY,
        }
    }

    /// Display `msg` and reset the decay timer.
    pub fn set(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.last_set = Some(Instant::now());
    }

    /// If the decay has elapsed, revert to the default message.
    pub fn tick(&mut self) {
        if let Some(t) = self.last_set {
            if t.elapsed() >= self.decay {
                self.message = self.default_message.clone();
                self.last_set = None;
            }
        }
    }

    /// Returns true if the current message is the default.
    pub fn is_default(&self) -> bool {
        self.last_set.is_none()
    }

    /// Returns the current message text.
    pub fn current(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_default() {
        let s = StatusBar::new("default");
        assert!(s.is_default());
        assert_eq!(s.current(), "");
    }

    #[test]
    fn test_status_bar_set() {
        let mut s = StatusBar::new("default");
        s.set("hello");
        assert!(!s.is_default());
        assert_eq!(s.current(), "hello");
    }

    #[test]
    fn test_status_bar_decay() {
        let mut s = StatusBar::new("default");
        s.decay = Duration::from_millis(1);
        s.set("hello");
        std::thread::sleep(Duration::from_millis(10));
        s.tick();
        assert!(s.is_default());
        assert_eq!(s.current(), "default");
    }
}
