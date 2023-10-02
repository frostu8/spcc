use bevy::prelude::*;

use spcc::AppState;

use spcc::stage::{StageLoader, StageBuilder};
use spcc::enemy::{Checkpoint, Follower, EnemyBundle};
use spcc::stats::{Stat as _, stat};

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            #[cfg(feature = "debug")]
            WorldInspectorPlugin::new(),
            spcc::stage::StagePlugin,
            spcc::enemy::path::PathPlugin,
            spcc::stats::StatPlugin,
            spcc::tile_map::GridPlugin,
            spcc::material::MaterialPlugin,
        ))
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .run();
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stage_loader: ResMut<StageLoader>,
    mut app_state: ResMut<NextState<AppState>>
) {
    // create camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, -8.0).looking_at(-Vec3::Z, Vec3::Y),
        ..default()
    });

    // FIXME: test enemy
    commands
        .spawn(EnemyBundle {
            follower: Follower::new([
                Checkpoint::at(Vec2::ZERO), Checkpoint::at(Vec2::Y * 2.0),
                Checkpoint::at(Vec2::ONE * 2.0), Checkpoint::at(Vec2::X * 2.0),
                Checkpoint::at(Vec2::ZERO),
            ]),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(shape::UVSphere::default().into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::RED,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                });

            parent
                .spawn(SpatialBundle::default())
                .insert(stat::MoveSpeed::modif().add(0.5));
        });

    // begin stage loading
    stage_loader.load(StageBuilder::new("maps/ccmap.ron"));

    // start setup
    app_state.set(AppState::StageLoading);
}