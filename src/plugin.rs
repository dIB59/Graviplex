//! Plugin system for extending the Graviplex engine.
//!
//! Plugins allow you to add optional functionality to your game without
//! modifying the core engine. Built-in plugins include:
//!
//! - [`FpsPlugin`] - Display FPS counter with configurable detail level
//!
//! # Example
//!
//! ```ignore
//! use graviplex::prelude::*;
//! use graviplex::plugins::FpsPlugin;
//!
//! App::build(MyGame::new())
//!     .add_plugin(FpsPlugin::default())
//!     .run()
//!     .unwrap();
//! ```
//!
//! # Creating Custom Plugins
//!
//! ```ignore
//! use graviplex::prelude::*;
//! use graviplex::plugin::Plugin;
//!
//! struct DebugGridPlugin {
//!     spacing: f32,
//!     color: Color,
//! }
//!
//! impl Plugin for DebugGridPlugin {
//!     fn name(&self) -> &'static str {
//!         "DebugGrid"
//!     }
//!
//!     fn render(&self, world: &World, draw: &mut DrawContext) {
//!         // Draw debug grid...
//!     }
//! }
//! ```

use crate::core::stats::{FpsCounter, FpsDisplayConfig};
use crate::ecs::World;
use crate::renderer::DrawContext;
use crate::Time;

/// Trait for engine plugins.
///
/// Plugins can hook into various parts of the game loop to add functionality.
/// All methods have default no-op implementations, so you only need to
/// implement the ones you need.
pub trait Plugin: 'static {
    /// Returns the name of this plugin (for debugging).
    fn name(&self) -> &'static str;

    /// Called once when the application starts.
    ///
    /// Use this to initialize plugin state or spawn entities.
    fn init(&mut self, _world: &mut World) {}

    /// Called every frame before rendering.
    ///
    /// Use this to update plugin state.
    fn update(&mut self, _world: &mut World, _time: &Time) {}

    /// Called every frame to render plugin visuals.
    ///
    /// Use this to draw debug overlays, grids, etc.
    fn render(&self, _world: &World, _draw: &mut DrawContext) {}

    /// Called to render plugin GUI elements (requires `gui` feature).
    ///
    /// The FPS counter and fps display info are provided for convenience.
    #[cfg(feature = "gui")]
    fn gui(&self, _ctx: &egui::Context, _fps: &FpsCounter) {}
}

/// FPS counter plugin that displays performance statistics.
///
/// This plugin renders an FPS counter overlay using egui. You can customize
/// what information is displayed using [`FpsDisplayConfig`].
///
/// # Example
///
/// ```ignore
/// use graviplex::prelude::*;
/// use graviplex::plugins::FpsPlugin;
///
/// // Simple FPS display
/// App::build(MyGame::new())
///     .add_plugin(FpsPlugin::default())
///     .run();
///
/// // Detailed FPS display
/// App::build(MyGame::new())
///     .add_plugin(FpsPlugin::detailed())
///     .run();
///
/// // Custom configuration
/// App::build(MyGame::new())
///     .add_plugin(FpsPlugin::new(
///         FpsDisplayConfig::default()
///             .with_frame_time()
///             .with_range()
///             .at_position(20.0, 20.0)
///     ))
///     .run();
/// ```
#[cfg(feature = "gui")]
pub struct FpsPlugin {
    config: FpsDisplayConfig,
}

#[cfg(feature = "gui")]
impl Default for FpsPlugin {
    fn default() -> Self {
        Self {
            config: FpsDisplayConfig::default(),
        }
    }
}

#[cfg(feature = "gui")]
impl FpsPlugin {
    /// Create an FPS plugin with custom configuration.
    pub fn new(config: FpsDisplayConfig) -> Self {
        Self { config }
    }

    /// Create a minimal FPS display (just FPS number).
    pub fn minimal() -> Self {
        Self {
            config: FpsDisplayConfig::minimal(),
        }
    }

    /// Create a detailed FPS display (FPS, frame time, min/max).
    pub fn detailed() -> Self {
        Self {
            config: FpsDisplayConfig::detailed(),
        }
    }

    /// Create a full FPS display (all statistics).
    pub fn full() -> Self {
        Self {
            config: FpsDisplayConfig::full(),
        }
    }
}

#[cfg(feature = "gui")]
impl Plugin for FpsPlugin {
    fn name(&self) -> &'static str {
        "FpsPlugin"
    }

    fn gui(&self, ctx: &egui::Context, fps: &FpsCounter) {
        let config = &self.config;

        // Don't show anything if all options are disabled
        if !config.show_fps
            && !config.show_frame_time
            && !config.show_range
            && !config.show_one_percent_low
        {
            return;
        }

        egui::Area::new(egui::Id::new("fps_plugin_counter"))
            .fixed_pos(egui::pos2(config.position.0, config.position.1))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                    .inner_margin(egui::Margin::same(6))
                    .corner_radius(egui::CornerRadius::same(4))
                    .show(ui, |ui| {
                        ui.set_min_width(80.0);

                        if config.show_fps {
                            ui.label(
                                egui::RichText::new(format!("FPS: {:.0}", fps.fps()))
                                    .color(fps_color(fps.fps()))
                                    .strong(),
                            );
                        }

                        if config.show_frame_time {
                            ui.label(
                                egui::RichText::new(format!("{:.2} ms", fps.frame_time_ms()))
                                    .color(egui::Color32::LIGHT_GRAY)
                                    .small(),
                            );
                        }

                        if config.show_range {
                            ui.label(
                                egui::RichText::new(format!("↓{:.0} ↑{:.0}", fps.min(), fps.max()))
                                    .color(egui::Color32::GRAY)
                                    .small(),
                            );
                        }

                        if config.show_one_percent_low {
                            ui.label(
                                egui::RichText::new(format!("1%: {:.0}", fps.one_percent_low()))
                                    .color(egui::Color32::from_rgb(255, 200, 100))
                                    .small(),
                            );
                        }
                    });
            });
    }
}

/// Get a color for the FPS value (green = good, yellow = ok, red = bad).
#[cfg(feature = "gui")]
fn fps_color(fps: f32) -> egui::Color32 {
    if fps >= 55.0 {
        egui::Color32::from_rgb(100, 255, 100) // Green
    } else if fps >= 30.0 {
        egui::Color32::from_rgb(255, 255, 100) // Yellow
    } else {
        egui::Color32::from_rgb(255, 100, 100) // Red
    }
}

/// A collection of plugins that can be added to the app.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Create an empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a plugin to the registry.
    pub fn add<P: Plugin>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// Initialize all plugins.
    pub fn init(&mut self, world: &mut World) {
        for plugin in &mut self.plugins {
            plugin.init(world);
        }
    }

    /// Update all plugins.
    pub fn update(&mut self, world: &mut World, time: &Time) {
        for plugin in &mut self.plugins {
            plugin.update(world, time);
        }
    }

    /// Render all plugins.
    pub fn render(&self, world: &World, draw: &mut DrawContext) {
        for plugin in &self.plugins {
            plugin.render(world, draw);
        }
    }

    /// Render GUI for all plugins.
    #[cfg(feature = "gui")]
    pub fn gui(&self, ctx: &egui::Context, fps: &FpsCounter) {
        for plugin in &self.plugins {
            plugin.gui(ctx, fps);
        }
    }

    /// Check if the registry has any plugins.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Get the number of plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }
}
