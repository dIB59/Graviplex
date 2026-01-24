//! Performance statistics for the application loop
//!
//! This module provides:
//! - [`AppStats`] - Final statistics at application exit
//! - [`FpsCounter`] - Real-time FPS tracking with history

use std::collections::VecDeque;

/// Statistics collected during the application run.
#[derive(Debug, Clone, Copy)]
pub struct AppStats {
    /// Average frames per second over the entire run.
    pub average_fps: f32,
    /// Total number of frames rendered.
    pub frame_count: u32,
    /// Total elapsed time in seconds.
    pub total_time: f32,
}

impl Default for AppStats {
    fn default() -> Self {
        Self {
            average_fps: 0.0,
            frame_count: 0,
            total_time: 0.0,
        }
    }
}

/// A proper FPS counter with rolling window statistics and history.
///
/// Tracks FPS using a rolling window of frame times, providing accurate
/// current, minimum, maximum, and average FPS values. Also maintains a
/// history buffer for visualization.
///
/// # Example
///
/// ```
/// use graviplex::core::stats::FpsCounter;
///
/// let mut fps = FpsCounter::new();
///
/// // Each frame, record the delta time
/// let dt = 0.016; // 16ms = ~60 FPS
/// fps.update(dt);
///
/// println!("FPS: {:.0} (min: {:.0}, max: {:.0})", fps.fps(), fps.min(), fps.max());
/// println!("Frame time: {:.2}ms", fps.frame_time_ms());
/// ```
#[derive(Debug, Clone)]
pub struct FpsCounter {
    /// Rolling window of frame times (in seconds)
    frame_times: VecDeque<f32>,
    /// Maximum size of the rolling window
    window_size: usize,
    /// History of FPS values for graphs
    history: VecDeque<f32>,
    /// Maximum history size
    history_size: usize,
    /// Cached current FPS (updated each frame)
    current_fps: f32,
    /// Minimum FPS in the rolling window
    min_fps: f32,
    /// Maximum FPS in the rolling window
    max_fps: f32,
    /// 1% low FPS (worst 1% of frames)
    one_percent_low: f32,
    /// Sum of all frame times for average calculation
    total_time: f64,
    /// Total frame count
    total_frames: u64,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    /// Create a new FPS counter with default settings.
    ///
    /// Uses a 100-frame rolling window and 300-frame history buffer.
    pub fn new() -> Self {
        Self::with_window_size(100)
    }

    /// Create an FPS counter with a custom rolling window size.
    ///
    /// Larger windows give more stable readings but respond slower to changes.
    pub fn with_window_size(window_size: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(window_size),
            window_size,
            history: VecDeque::with_capacity(300),
            history_size: 300,
            current_fps: 0.0,
            min_fps: f32::MAX,
            max_fps: 0.0,
            one_percent_low: 0.0,
            total_time: 0.0,
            total_frames: 0,
        }
    }

    /// Set the history buffer size for FPS graphing.
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self.history = VecDeque::with_capacity(size);
        self
    }

    /// Update the counter with the current frame's delta time.
    ///
    /// Call this once per frame with the time elapsed since the last frame.
    pub fn update(&mut self, delta_time: f32) {
        // Skip invalid frame times
        if delta_time <= 0.0 || delta_time > 1.0 {
            return;
        }

        // Add to rolling window
        if self.frame_times.len() >= self.window_size {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(delta_time);

        // Track totals
        self.total_time += delta_time as f64;
        self.total_frames += 1;

        // Recalculate statistics from the rolling window
        self.recalculate_stats();

        // Add to history
        if self.history.len() >= self.history_size {
            self.history.pop_front();
        }
        self.history.push_back(self.current_fps);
    }

    /// Recalculate all statistics from the rolling window.
    fn recalculate_stats(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        // Calculate average frame time
        let sum: f32 = self.frame_times.iter().sum();
        let avg_frame_time = sum / self.frame_times.len() as f32;
        self.current_fps = 1.0 / avg_frame_time;

        // Find min/max (convert frame times to FPS)
        let mut fps_values: Vec<f32> = self
            .frame_times
            .iter()
            .filter(|&&dt| dt > 0.0)
            .map(|&dt| 1.0 / dt)
            .collect();

        if !fps_values.is_empty() {
            // Sort for percentile calculations
            fps_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            self.min_fps = fps_values[0];
            self.max_fps = fps_values[fps_values.len() - 1];

            // 1% low - average of the worst 1% of frames (or at least 1 frame)
            let one_percent_count = (fps_values.len() / 100).max(1);
            let one_percent_sum: f32 = fps_values.iter().take(one_percent_count).sum();
            self.one_percent_low = one_percent_sum / one_percent_count as f32;
        }
    }

    /// Get the current smoothed FPS.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.current_fps
    }

    /// Get the current frame time in seconds.
    #[inline]
    pub fn frame_time(&self) -> f32 {
        if self.current_fps > 0.0 {
            1.0 / self.current_fps
        } else {
            0.0
        }
    }

    /// Get the current frame time in milliseconds.
    #[inline]
    pub fn frame_time_ms(&self) -> f32 {
        self.frame_time() * 1000.0
    }

    /// Get the minimum FPS in the rolling window.
    #[inline]
    pub fn min(&self) -> f32 {
        if self.min_fps == f32::MAX {
            0.0
        } else {
            self.min_fps
        }
    }

    /// Get the maximum FPS in the rolling window.
    #[inline]
    pub fn max(&self) -> f32 {
        self.max_fps
    }

    /// Get the 1% low FPS (average of worst 1% of frames).
    ///
    /// This is a better indicator of stuttering than min FPS.
    #[inline]
    pub fn one_percent_low(&self) -> f32 {
        self.one_percent_low
    }

    /// Get the average FPS over the entire runtime.
    pub fn average(&self) -> f32 {
        if self.total_time > 0.0 {
            self.total_frames as f32 / self.total_time as f32
        } else {
            0.0
        }
    }

    /// Get the total frame count.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.total_frames
    }

    /// Get the FPS history for graphing.
    ///
    /// Returns an iterator over recent FPS values, oldest first.
    pub fn history(&self) -> impl Iterator<Item = f32> + '_ {
        self.history.iter().copied()
    }

    /// Get the history as a slice (for compatibility with plotting libraries).
    pub fn history_vec(&self) -> Vec<f32> {
        self.history.iter().copied().collect()
    }

    /// Reset all statistics.
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.history.clear();
        self.current_fps = 0.0;
        self.min_fps = f32::MAX;
        self.max_fps = 0.0;
        self.one_percent_low = 0.0;
        self.total_time = 0.0;
        self.total_frames = 0;
    }
}

/// Configuration for FPS display in the GUI.
#[derive(Debug, Clone, Copy)]
pub struct FpsDisplayConfig {
    /// Show current FPS
    pub show_fps: bool,
    /// Show frame time in milliseconds
    pub show_frame_time: bool,
    /// Show min/max FPS range
    pub show_range: bool,
    /// Show 1% low FPS
    pub show_one_percent_low: bool,
    /// Position from top-left corner
    pub position: (f32, f32),
}

impl Default for FpsDisplayConfig {
    fn default() -> Self {
        Self {
            show_fps: true,
            show_frame_time: false,
            show_range: false,
            show_one_percent_low: false,
            position: (10.0, 10.0),
        }
    }
}

impl FpsDisplayConfig {
    /// Create a minimal config showing only FPS.
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Create a detailed config showing FPS, frame time, and range.
    pub fn detailed() -> Self {
        Self {
            show_fps: true,
            show_frame_time: true,
            show_range: true,
            show_one_percent_low: false,
            position: (10.0, 10.0),
        }
    }

    /// Create a full config showing all statistics.
    pub fn full() -> Self {
        Self {
            show_fps: true,
            show_frame_time: true,
            show_range: true,
            show_one_percent_low: true,
            position: (10.0, 10.0),
        }
    }

    /// Enable frame time display.
    pub fn with_frame_time(mut self) -> Self {
        self.show_frame_time = true;
        self
    }

    /// Enable min/max range display.
    pub fn with_range(mut self) -> Self {
        self.show_range = true;
        self
    }

    /// Enable 1% low display.
    pub fn with_one_percent_low(mut self) -> Self {
        self.show_one_percent_low = true;
        self
    }

    /// Set the display position.
    pub fn at_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Disable FPS display entirely.
    pub fn hidden() -> Self {
        Self {
            show_fps: false,
            show_frame_time: false,
            show_range: false,
            show_one_percent_low: false,
            position: (10.0, 10.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_counter_basic() {
        let mut counter = FpsCounter::new();

        // Simulate 60 FPS (16.67ms per frame)
        for _ in 0..100 {
            counter.update(1.0 / 60.0);
        }

        // Should be approximately 60 FPS
        assert!((counter.fps() - 60.0).abs() < 1.0);
        assert!(counter.frame_count() == 100);
    }

    #[test]
    fn test_fps_counter_variable_framerate() {
        let mut counter = FpsCounter::with_window_size(10);

        // Mix of fast and slow frames
        for _ in 0..5 {
            counter.update(1.0 / 60.0); // 60 FPS
        }
        for _ in 0..5 {
            counter.update(1.0 / 30.0); // 30 FPS
        }

        // Min should be around 30, max around 60
        assert!(counter.min() >= 29.0 && counter.min() <= 31.0);
        assert!(counter.max() >= 59.0 && counter.max() <= 61.0);
    }

    #[test]
    fn test_fps_counter_frame_time() {
        let mut counter = FpsCounter::new();
        counter.update(0.016); // ~60 FPS

        // Frame time should be reported correctly
        assert!((counter.frame_time_ms() - 16.0).abs() < 1.0);
    }

    #[test]
    fn test_fps_counter_history() {
        let mut counter = FpsCounter::new().with_history_size(50);

        for i in 0..100 {
            counter.update(0.016 + (i as f32 * 0.0001));
        }

        // History should be capped at 50
        assert_eq!(counter.history_vec().len(), 50);
    }

    #[test]
    fn test_fps_counter_reset() {
        let mut counter = FpsCounter::new();
        counter.update(0.016);
        counter.update(0.016);

        counter.reset();

        assert_eq!(counter.fps(), 0.0);
        assert_eq!(counter.frame_count(), 0);
        assert!(counter.history_vec().is_empty());
    }

    #[test]
    fn test_fps_counter_ignores_invalid() {
        let mut counter = FpsCounter::new();
        counter.update(0.016);
        counter.update(-1.0); // Invalid - negative
        counter.update(0.0); // Invalid - zero
        counter.update(2.0); // Invalid - too large

        assert_eq!(counter.frame_count(), 1);
    }

    #[test]
    fn test_fps_display_config() {
        let config = FpsDisplayConfig::detailed();
        assert!(config.show_fps);
        assert!(config.show_frame_time);
        assert!(config.show_range);
        assert!(!config.show_one_percent_low);

        let full = FpsDisplayConfig::full();
        assert!(full.show_one_percent_low);

        let hidden = FpsDisplayConfig::hidden();
        assert!(!hidden.show_fps);
    }
}
