use std::time::Duration;

use bevy::app::Plugin;
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};

use crate::particle::Particle;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(
            AutomaticUpdate::<Particle>::new()
                .with_frequency(Duration::from_secs_f32(0.3))
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_transform(TransformMode::GlobalTransform),
        );
    }
}
