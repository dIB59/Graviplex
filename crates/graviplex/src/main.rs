//! Graviplex - N-Body Simulation Binary

use graviplex::NBodyGame;
use graviplex_engine::App;

fn main() -> Result<(), winit::error::EventLoopError> {
    // 256K particles (1 << 18)
    let game = NBodyGame::new(1 << 18);
    App::new(game).run()
}
