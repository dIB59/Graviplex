//! Graviplex - N-Body Simulation Binary

use graviplex::App;
use graviplex_sim::NBodyGame;

fn main() -> Result<(), winit::error::EventLoopError> {
    // 1M particles (1 << 20)
    let game = NBodyGame::new(1 << 20);
    App::new(game).run()
}
