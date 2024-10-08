use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::movement::Velocity;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_systems(Startup, setup_fps_counter);
        app.add_systems(Startup, setup_entity_count);
        app.add_systems(Update, (fps_text_update_system, fps_counter_showhide));
        app.add_systems(Update, entity_count_update_system);
    }
}

#[derive(Component)]
struct FpsRoot;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct EntityCountRoot;

#[derive(Component)]
struct EntityCountText;

fn setup_fps_counter(mut commands: Commands) {
    let root = commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                bottom: Val::Auto,
                left: Val::Auto,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsRoot)
        .id();
    let text_fps = commands
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
            ]),
            ..Default::default()
        })
        .insert(FpsText)
        .id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{value:>4.0}");

            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::srgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::srgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::srgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                // Below 30 FPS, use red color
                Color::srgb(1.0, 0.0, 0.0)
            }
        } else {
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

fn setup_entity_count(mut commands: Commands) {
    let root = commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Percent(1.0),
                top: Val::Percent(4.3),
                bottom: Val::Auto,
                left: Val::Auto,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(EntityCountRoot)
        .id();
    let entity_count = commands
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "ENT: ".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
            ]),
            ..Default::default()
        })
        .insert(EntityCountText)
        .id();
    commands.entity(root).push_children(&[entity_count]);
}

fn entity_count_update_system(
    mut query: ParamSet<(Query<&mut Text, With<EntityCountText>>,)>,
    query2: Query<(Entity, &Transform, &Velocity)>,
) {
    let some = query2.iter().count();
    let mut text_query = query.p0();
    for mut text in text_query.iter_mut() {
        text.sections[1].value = format!("{some:>4.0}");
        text.sections[1].style.color = Color::WHITE;
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        trace!("FPS {:?}", q.single());
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
