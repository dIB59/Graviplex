//! Performance statistics for the application loop

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
