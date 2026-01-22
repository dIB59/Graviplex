//! End-to-end tests for Graviplex engine.
//!
//! Note: These tests verify the API works correctly at a high level.

use graviplex::prelude::*;
use graviplex::advanced::CircleInstance;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);

        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(-2.0, -2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
        assert!((a.length() - 2.236).abs() < 0.01);
    }

    #[test]
    fn test_circle_geometry() {
        let c = Circle::new(Vec2::ZERO, 10.0, Color::RED);

        assert!(c.contains(Vec2::new(5.0, 5.0)));
        assert!(!c.contains(Vec2::new(15.0, 0.0)));

        let other = Circle::new(Vec2::new(15.0, 0.0), 10.0, Color::BLUE);
        assert!(c.intersects(&other));

        let far = Circle::new(Vec2::new(100.0, 0.0), 10.0, Color::GREEN);
        assert!(!c.intersects(&far));
    }

    #[test]
    fn test_rect_geometry() {
        let r = Rect::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

        assert!(r.contains(Vec2::new(5.0, 5.0)));
        assert!(!r.contains(Vec2::new(15.0, 5.0)));
        assert_eq!(r.center(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_color_creation() {
        let c = Color::rgba(1.0, 0.5, 0.25, 1.0);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.25);
        assert_eq!(c.a, 1.0);

        let from_arr: Color = [1.0, 0.0, 0.0, 1.0].into();
        assert_eq!(from_arr, Color::RED);
    }

    #[test]
    fn test_circle_instance_creation() {
        let c = CircleInstance::new([10.0, 20.0], 5.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(c.position, [10.0, 20.0]);
        assert_eq!(c.radius, 5.0);

        // Test From<Circle>
        let circle = Circle::new(Vec2::new(30.0, 40.0), 15.0, Color::BLUE);
        let instance: CircleInstance = circle.into();
        assert_eq!(instance.position, [30.0, 40.0]);
        assert_eq!(instance.radius, 15.0);
    }

    #[test]
    fn test_app_stats_default() {
        let stats = AppStats::default();
        assert_eq!(stats.average_fps, 0.0);
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.total_time, 0.0);
    }

    #[test]
    fn test_camera_config_builder() {
        let config = CameraConfig::centered()
            .with_scale(100.0)
            .with_position(50.0, 50.0)
            .with_zoom_speed(1.5)
            .with_move_speed(500.0)
            .with_zoom_range(0.1, 100.0);

        assert_eq!(config.scale, 100.0);
        assert_eq!(config.position, [50.0, 50.0]);
        assert_eq!(config.zoom_speed, 1.5);
        assert_eq!(config.move_speed, 500.0);
        assert_eq!(config.min_zoom, 0.1);
        assert_eq!(config.max_zoom, 100.0);
    }

    #[test]
    fn test_time_tracking() {
        let mut time = Time::new();

        // Initial state
        assert_eq!(time.frame_count(), 0);
        assert_eq!(time.delta(), 0.0);

        // After one update
        std::thread::sleep(std::time::Duration::from_millis(16));
        time.update();

        assert_eq!(time.frame_count(), 1);
        assert!(time.delta() > 0.0);
        assert!(time.elapsed() > 0.0);
    }
}
