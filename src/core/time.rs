//! Frame timing and FPS tracking.
//!
//! The [`Time`] struct provides frame timing information and is passed to
//! `GameLoop::update()` each frame.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use crate::core::stats::FpsCounter;

static GLOBAL_FPS: AtomicU32 = AtomicU32::new(0);
static GLOBAL_FRAME_TIME: AtomicU32 = AtomicU32::new(0);

/// Returns the current smoothed FPS (global accessor).
///
/// **Deprecated**: Use `time.fps()` from the `Time` reference passed to `update()` instead.
#[deprecated(since = "0.3.0", note = "Use time.fps() from GameLoop::update() instead")]
pub fn get_fps() -> f32 {
    GLOBAL_FPS.load(Ordering::Relaxed) as f32 / 10.0
}

/// Returns the time difference between current and last frame in seconds (global accessor).
///
/// **Deprecated**: Use `time.delta()` from the `Time` reference passed to `update()` instead.
#[deprecated(since = "0.3.0", note = "Use time.delta() from GameLoop::update() instead")]
pub fn get_frame_time() -> f32 {
    GLOBAL_FRAME_TIME.load(Ordering::Relaxed) as f32 / 10000.0
}

/// Frame timing information.
///
/// Provides access to delta time, FPS, and elapsed time.
/// An instance is passed to `GameLoop::update()` each frame.
///
/// # Example
///
/// ```ignore
/// fn update(&mut self, time: &Time, _gfx: &Graphics) {
///     // Move player based on delta time
///     self.player.position += self.player.velocity * time.delta();
///     
///     // Log performance
///     if time.frame_count() % 60 == 0 {
///         println!("FPS: {:.1}", time.fps());
///     }
/// }
/// ```
pub struct Time {
    start_time: Instant,
    last_frame: Instant,
    delta: f32,
    fps_counter: FpsCounter,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta: 0.0,
            fps_counter: FpsCounter::new(),
        }
    }
}

impl Time {
    /// Create a new Time tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update timing (called by the engine each frame).
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Update the FPS counter
        self.fps_counter.update(self.delta);

        // Update globals for backwards compatibility
        GLOBAL_FPS.store((self.fps_counter.fps() * 10.0) as u32, Ordering::Relaxed);
        GLOBAL_FRAME_TIME.store((self.delta * 10000.0) as u32, Ordering::Relaxed);
    }

    /// Time since last frame in seconds.
    ///
    /// Use this for frame-rate independent movement and physics.
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Current smoothed frames per second.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.fps_counter.fps()
    }

    /// Total frames rendered since the application started.
    #[inline]
    pub fn frame_count(&self) -> u32 {
        self.fps_counter.frame_count() as u32
    }

    /// Average FPS over the entire run.
    pub fn average_fps(&self) -> f32 {
        self.fps_counter.average()
    }

    /// Total elapsed time in seconds since the Time struct was created.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    /// Get the FPS counter for detailed statistics.
    ///
    /// The FPS counter provides min/max, 1% low, frame time in ms, and history.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let fps = time.fps_counter();
    /// println!("FPS: {:.0} (min: {:.0}, max: {:.0})", fps.fps(), fps.min(), fps.max());
    /// println!("Frame time: {:.2}ms", fps.frame_time_ms());
    /// println!("1% low: {:.0}", fps.one_percent_low());
    /// ```
    #[inline]
    pub fn fps_counter(&self) -> &FpsCounter {
        &self.fps_counter
    }

    /// Get the current frame time in milliseconds.
    #[inline]
    pub fn frame_time_ms(&self) -> f32 {
        self.delta * 1000.0
    }

    /// Get the minimum FPS from the rolling window.
    #[inline]
    pub fn fps_min(&self) -> f32 {
        self.fps_counter.min()
    }

    /// Get the maximum FPS from the rolling window.
    #[inline]
    pub fn fps_max(&self) -> f32 {
        self.fps_counter.max()
    }

    /// Get the 1% low FPS (average of worst 1% of frames).
    ///
    /// This is a better indicator of stuttering than minimum FPS.
    #[inline]
    pub fn fps_one_percent_low(&self) -> f32 {
        self.fps_counter.one_percent_low()
    }
}
