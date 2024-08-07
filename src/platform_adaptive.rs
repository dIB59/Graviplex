#![allow(unused_imports)]

use bevy::app::*;
use bevy::render::settings::*;
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;

pub struct PlatformAdaptiveDefaultPlugin;

impl Plugin for PlatformAdaptiveDefaultPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "windows")]
        app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..Default::default()
            }),
            ..Default::default()
        }));
        #[cfg(target_os = "macos")]
        app.add_plugins(DefaultPlugins);
        #[cfg(target_os = "linux")]
        app.add_plugins(DefaultPlugins);
    }
}
