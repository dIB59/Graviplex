use bevy::app::*;
use bevy::DefaultPlugins;
use bevy::render::RenderPlugin;
use bevy::render::settings::*;

pub struct CustomRenderPlugin;

impl Plugin for CustomRenderPlugin {
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
    }
}
