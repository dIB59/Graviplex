use std::time::Instant;

pub struct Time {
    last_frame: Instant,
    delta: f32,
    fps: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: 0.0,
            fps: 0.0,
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

        if self.delta > 0.0 {
            let current_fps = 1.0 / self.delta;
            self.fps = self.fps * 0.9 + current_fps * 0.1;
        }
    }

    /// Gives time diffrence between current and last frame
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Returns the current smoothed FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }
}
