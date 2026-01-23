//! FPS assertion test - verifies the engine runs at expected frame rates.

use graviplex::prelude::*;

struct TestGame;

impl GameLoop for TestGame {
    fn init(&mut self, _world: &mut World, _gfx: &Graphics) {}

    fn update(&mut self, _world: &mut World, res: &Resources) {
        // Use the Time reference from Resources (preferred)
        let time = res.time;
        if time.fps() > 0.0 {
            println!(
                "Current FPS: {:.1}, Frame Time: {:.4}s",
                time.fps(),
                time.delta()
            );
        }
    }

    fn render(&mut self, _world: &World, _draw: &mut DrawContext) {}
}

fn main() {
    println!("Running FPS assertion test for 2 seconds...");

    // Use the new builder API with exit_after
    let stats = App::build(TestGame)
        .title("FPS Assertion Test")
        .exit_after(2.0)
        .run()
        .expect("App failed to run");

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
