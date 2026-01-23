//! API demonstration example showing the new consolidated API.

use graviplex::prelude::*;
use graviplex::advanced::CircleInstance;

struct ApiTestGame;

impl GameLoop for ApiTestGame {
    fn init(&mut self, _world: &mut World, _gfx: &Graphics) {}
    fn update(&mut self, _world: &mut World, _res: &Resources) {}

    fn render(&mut self, _world: &World, draw: &mut DrawContext) {
        // Draw circles using rich domain types
        let c1 = Circle::new(Vec2::new(-200.0, 0.0), 50.0, Color::RED);
        draw.circle(c1);

        // Draw using tuples with engine types
        draw.circle((Vec2::new(0.0, 0.0), 75.0, Color::GREEN));

        // Draw using raw arrays
        draw.circle(([200.0, 0.0], 50.0, [0.0, 0.0, 1.0, 1.0]));

        // Draw lines using tuples
        draw.line(([-500.0, -500.0], [500.0, 500.0], [1.0, 1.0, 1.0, 0.5]));
        draw.line(([-500.0, 500.0], [500.0, -500.0], [1.0, 1.0, 1.0, 0.5]));

        // Batch draw many small circles using CircleInstance
        let mut batch = Vec::new();
        for i in 0..100 {
            let angle = (i as f32) * 0.1;
            let x = angle.cos() * 300.0;
            let y = angle.sin() * 300.0;
            batch.push(CircleInstance::new([x, y], 5.0, [1.0, 1.0, 0.0, 1.0]));
        }
        draw.circles(&batch);
    }
}

fn main() {
    App::build(ApiTestGame)
        .title("Graviplex API Test")
        .size(1200, 800)
        .run()
        .unwrap();
}
