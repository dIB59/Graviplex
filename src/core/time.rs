use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

static GLOBAL_FPS: AtomicU32 = AtomicU32::new(0);
static GLOBAL_FRAME_TIME: AtomicU32 = AtomicU32::new(0);

/// Returns the current smoothed FPS.
pub fn get_fps() -> f32 {
    GLOBAL_FPS.load(Ordering::Relaxed) as f32 / 10.0
}

/// Returns the time difference between current and last frame in seconds.
pub fn get_frame_time() -> f32 {
    GLOBAL_FRAME_TIME.load(Ordering::Relaxed) as f32 / 10000.0
}

pub struct Time {
    start_time: Instant,
    last_frame: Instant,
    delta: f32,
    fps: f32,
    total_frames: u32,
    sum_fps: f64,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta: 0.0,
            fps: 0.0,
            total_frames: 0,
            sum_fps: 0.0,
        }
    }
}

impl Time {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates delta to be the difference (in seconds) since last frame and now.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.total_frames += 1;

        if self.delta > 0.0 {
            let current_fps = 1.0 / self.delta;
            self.fps = self.fps * 0.9 + current_fps * 0.1;
            self.sum_fps += current_fps as f64;
        }

        // Update globals (scaled for atomic storage)
        GLOBAL_FPS.store((self.fps * 10.0) as u32, Ordering::Relaxed);
        GLOBAL_FRAME_TIME.store((self.delta * 10000.0) as u32, Ordering::Relaxed);
    }

    /// Gives time diffrence between current and last frame
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Returns the current smoothed FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the total frames rendered.
    pub fn frame_count(&self) -> u32 {
        self.total_frames
    }

    /// Returns the average FPS since creation.
    pub fn average_fps(&self) -> f32 {
        if self.total_frames > 0 {
            (self.sum_fps / self.total_frames as f64) as f32
        } else {
            0.0
        }
    }

    /// Returns the total elapsed time in seconds since the Time struct was created.
    pub fn elapsed(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}
