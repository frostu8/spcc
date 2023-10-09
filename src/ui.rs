//! Core UI and display functionality.

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy::ui::UiSystem;

use crate::damage::Health;

/// The core UI plugin.
pub struct CoreUiPlugin;

impl Plugin for CoreUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                (
                    create_status_bar
                        .before(UiSystem::Layout),
                    sync_health_bar,
                    sync_status_bar_position
                        .after(TransformSystem::TransformPropagate)
                        .after(create_status_bar),
                ),
            );
    }
}

/// A UI status bar.
#[derive(Debug, Component, Clone)]
pub struct StatusBar {
    entity: Entity,
}

#[derive(Debug, Component, Clone)]
struct HealthBar {
    entity: Entity,
    percentage: f32,
    dampening: f32,
}

impl HealthBar {
    pub fn new(entity: Entity) -> HealthBar {
        HealthBar {
            entity,
            percentage: 1.0,
            dampening: 0.0,
        }
    }

    pub fn with_dampening(self, dampening: f32) -> HealthBar {
        HealthBar {
            dampening,
            ..self
        }
    }
}

fn sync_health_bar(
    mut health_bar_query: Query<(&mut HealthBar, &mut Style)>,
    health_query: Query<&Health>,
    time: Res<Time>,
) {
    for (mut health_bar, mut health_bar_style) in health_bar_query.iter_mut() {
        let Ok(health) = health_query.get(health_bar.entity) else {
            // invalid bar, should get cleaned up by `sync_status_bar_position`
            continue;
        };

        // do damping
        let distance = health.percentage() - health_bar.percentage;

        if distance.abs() < f32::EPSILON {
            // do not update to avoid NaN calculations below
            continue;
        }

        // dampening function
        let dampening = (distance.abs() + 0.1) * (1.0 / health_bar.dampening);
        let velocity = dampening * time.delta_seconds();
        let mut adjusted_distance = distance * velocity;

        if adjusted_distance.abs() > distance.abs() {
            adjusted_distance = distance;
        }

        health_bar.percentage += adjusted_distance;

        // apply percentage
        health_bar_style.width = Val::Percent(health_bar.percentage * 100.0);
    }
}

pub fn sync_status_bar_position(
    mut commands: Commands,
    mut query: Query<(Entity, &StatusBar, &mut Style)>,
    position_query: Query<&GlobalTransform>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
) {
    let (camera_transform, camera) = camera_query.single(); // TODO: lol

    for (status_bar_entity, status_bar, mut style) in query.iter_mut() {
        let Ok(entity_transform) = position_query.get(status_bar.entity) else {
            // delete status bar
            commands
                .entity(status_bar_entity)
                .despawn_recursive();

            continue;
        };

        // sync position
        let ndc = camera.world_to_ndc(camera_transform, entity_transform.translation());

        let Some(ndc) = ndc else {
            continue;
        };

        // convert ndc to position
        style.left = Val::Percent((ndc.x + 1.0) / 2.0 * 100.0);
        style.top = Val::Percent((-ndc.y + 1.0) / 2.0 * 100.0);
    }
}

/// Creates status bars for newly added [`Health`] components.
pub fn create_status_bar(
    mut commands: Commands,
    query: Query<Entity, Added<Health>>,
) {
    for entity in query.iter() {
        // create new health bar
        commands
            .spawn((
                NodeBundle::default(),
                StatusBar { entity },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        NodeBundle {
                            style: Style {
                                // TODO: how big should status bars be?
                                height: Val::Px(4.0),
                                width: Val::Px(48.0),
                                top: Val::Px(8.0),
                                left: Val::Px(-48.0 / 2.0),
                                ..default()
                            },
                            background_color: Color::BLACK.into(),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                NodeBundle {
                                    style: Style {
                                        height: Val::Percent(100.0),
                                        width: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    background_color: Color::RED.into(),
                                    z_index: ZIndex::Local(1),
                                    ..default()
                                },
                                HealthBar::new(entity),
                            ));

                        parent
                            .spawn((
                                NodeBundle {
                                    style: Style {
                                        height: Val::Percent(100.0),
                                        width: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    background_color: Color::WHITE.into(),
                                    ..default()
                                },
                                HealthBar::new(entity)
                                    .with_dampening(0.01),
                            ));
                    });
            });
    }
}
