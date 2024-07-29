use bevy::app::{App, Plugin};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};

pub struct VulkanRenderPlugin;

impl Plugin for VulkanRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
}