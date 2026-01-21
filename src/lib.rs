//! Graviplex Engine - A reusable 2D GPU-accelerated game engine
//!
//! This crate provides:
//! - GPU rendering context (wgpu)
//! - 2D camera with pan/zoom
//! - Input handling
//! - egui integration for GUI
//! - GameLoop trait for implementing games

pub mod core;
pub mod gui;
pub mod input;
pub mod renderer;

mod app;
mod game_loop;

pub use app::App;
pub use core::time::Time;
pub use game_loop::GameLoop;
pub use gui::{gui_renderer::UiPipeline, Gui};
pub use input::InputState;
pub use renderer::{
    Camera2D, CameraController, GpuContext, LineInstance, LinePipeline, RenderPipeline, Vertex,
};

// Re-export dependencies for convenience
pub use egui;
pub use wgpu;
pub use wgpu::include_wgsl;
pub use winit;
