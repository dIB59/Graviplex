use bevy::app::{App, Plugin};

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
    }
}
