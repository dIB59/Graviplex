use graviplex::*;

struct ApiTestGame;

impl GameLoop for ApiTestGame {
    fn init(&mut self, _gpu: &GpuContext) {}
    fn update(&mut self, _dt: f32, _gpu: &GpuContext) {}

    fn render(&mut self, draw: &mut DrawContext) {
        // Draw some circles
        draw.draw_circle([-200.0, 0.0], 50.0, [1.0, 0.0, 0.0, 1.0]);
        draw.draw_circle([0.0, 0.0], 75.0, [0.0, 1.0, 0.0, 1.0]);
        draw.draw_circle([200.0, 0.0], 50.0, [0.0, 0.0, 1.0, 1.0]);

        // Draw some lines
        draw.draw_line([-500.0, -500.0], [500.0, 500.0], [1.0, 1.0, 1.0, 0.5]);
        draw.draw_line([-500.0, 500.0], [500.0, -500.0], [1.0, 1.0, 1.0, 0.5]);

        // Batch draw many small circles
        let mut batch = Vec::new();
        for i in 0..100 {
            let angle = (i as f32) * 0.1;
            let x = angle.cos() * 300.0;
            let y = angle.sin() * 300.0;
            batch.push(CircleInstance {
                position: [x, y],
                radius: 5.0,
                color: [1.0, 1.0, 0.0, 1.0],
            });
        }
        draw.draw_circles(&batch);
    }
}

fn main() {
    let game = ApiTestGame;
    let app = App::new(game);
    app.run().unwrap();
}
