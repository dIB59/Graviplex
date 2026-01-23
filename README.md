# Graviplex

A 2D GPU-accelerated game engine with an immediate-mode drawing API, built with Rust and wgpu.

## Features

- **Immediate-mode Drawing** - Simple, Raylib-inspired API for drawing circles, lines, and more
- **GPU-accelerated** - Powered by wgpu for cross-platform graphics
- **High Performance** - Render 1M+ particles with GPU buffer support
- **Flexible Input** - Draw with rich domain types, tuples, or raw arrays
- **egui Integration** - Built-in GUI support (optional, feature-gated)
- **2D Camera** - Pan and zoom with built-in camera controls

## Quick Start

```rust
use graviplex::prelude::*;

struct MyGame;

impl GameLoop for MyGame {
    fn init(&mut self, _gfx: &Graphics) {}
    
    fn update(&mut self, time: &Time, _gfx: &Graphics) {
        println!("FPS: {:.1}", time.fps());
    }
    
    fn render(&mut self, draw: &mut DrawContext) {
        // Draw with rich types
        draw.circle(Circle::new(Vec2::ZERO, 50.0, Color::RED));
        
        // Or with tuples
        draw.circle((Vec2::new(100.0, 0.0), 30.0, Color::BLUE));
        
        // Or with raw arrays
        draw.line(([-100.0, 0.0], [100.0, 0.0], [1.0, 1.0, 1.0, 1.0]));
    }
}

fn main() {
    App::build(MyGame)
        .title("My Game")
        .size(1280, 720)
        .vsync(true)
        .run()
        .unwrap();
}
```

## App Configuration

```rust
App::build(game)
    .title("Window Title")
    .size(1280, 720)
    .vsync(false)
    .camera(CameraConfig::centered().with_scale(100.0))
    .exit_after(5.0)  // Auto-exit after 5 seconds (for testing)
    .run()
```

## High-Performance Rendering

For rendering large numbers of particles (100K+), use GPU buffers:

```rust
use graviplex::advanced::CircleInstance;
use wgpu::util::DeviceExt;

fn init(&mut self, gfx: &Graphics) {
    let instances: Vec<CircleInstance> = /* generate particles */;
    self.buffer = gfx.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Particles"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
    });
}

fn render(&mut self, draw: &mut DrawContext) {
    draw.circles_from_buffer(&self.buffer, 1_000_000);
}
```

## Feature Flags

```toml
[dependencies]
graviplex = "0.3"  # Default includes GUI

# Or without GUI:
graviplex = { version = "0.3", default-features = false }
```

| Feature | Default | Description |
|---------|---------|-------------|
| `gui` | ✓ | egui integration for UI |
| `physics` | | Physics simulation types |

## Demo Video

https://github.com/user-attachments/assets/998e33a2-57b9-49c4-8cee-fdb0158826f9

