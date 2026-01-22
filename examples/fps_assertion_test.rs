use graviplex::*;

struct TestGame;

impl GameLoop for TestGame {
    fn init(&mut self, _gpu: &GpuContext) {}
    fn update(&mut self, _dt: f32, _gpu: &GpuContext) {
        // You can use Raylib-style accessors anywhere
        let fps = get_fps();
        let frame_time = get_frame_time();
        if fps > 0.0 {
            println!("Current FPS: {:.1}, Frame Time: {:.4}s", fps, frame_time);
        }
    }
    fn render(&mut self, _draw: &mut DrawContext) {}
}

fn main() {
    println!("Running FPS assertion test for 2 seconds...");

    let game = TestGame;
    // We run for 2 seconds to get a good average
    let app = App::new(game).with_exit_time(2.0);

    let stats = app.run().expect("App failed to run");

    println!("\n--- Run Statistics ---");
    println!("Average FPS: {:.1}", stats.average_fps);
    println!("Total Frames: {}", stats.frame_count);
    println!("Total Time: {:.2}s", stats.total_time);

    // Assertions
    assert!(stats.frame_count > 0, "No frames were rendered!");
    assert!(
        stats.average_fps > 10.0,
        "FPS is suspiciously low: {:.1}",
        stats.average_fps
    );

    println!("\nTest passed! FPS is confirmed to be working and unlocked.");
}
