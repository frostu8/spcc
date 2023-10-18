use bevy::prelude::*;

use std::time::Duration;

use spcc::AppState;

use spcc::loader::{LoadStageEvent, StageBuilder};
use spcc::battle::{
    path::{Checkpoint, Follower},
    auto_attack::{AttackCycle, Melee},
    targeting::Range,
    EnemyBundle,
    EnemyStatBundle,
    OperatorBundle,
    OperatorStatBundle,
    StatBundle,
};
use spcc::tile_map::nav::NavBundle;
use spcc::tile_map::{Coordinates, Grid};
use spcc::stats::{Stat as _, stat};
//use spcc::effect::HpDecay;

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

//use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            //DefaultPickingPlugins,
            #[cfg(feature = "debug")]
            WorldInspectorPlugin::new(),
            spcc::loader::LoaderPlugin,
            spcc::battle::BattlePlugins,
            spcc::stats::StatPlugin,
            spcc::tile_map::GridPlugin,
            spcc::tile_map::nav::NavPlugin,
            spcc::material::MaterialPlugin,
            spcc::status::StatusPlugin,
            spcc::ui::UiPlugin,
            // DEBUG:
            spcc::battle::DebugDrawPlugin,
            //spcc::tile_map::focus::FocusPlugin,
        ))
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, setup_tile_map)
        .run();
}

pub fn setup_tile_map(
    mut debounce: Local<bool>,
    mut commands: Commands,
    query: Query<Entity, With<Grid>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *debounce {
        return;
    }

    let Ok(grid) = query.get_single() else {
        return;
    };

    *debounce = true;

    // FIXME: test operator
    commands
        .spawn((
            OperatorBundle {
                coordinates: Coordinates::new(6, 3),
                stats: OperatorStatBundle {
                    hp: StatBundle::new(stat::MaxHp::new(2200.0)),
                    def: StatBundle::new(stat::Def::new(280)),
                    atk_interval: StatBundle::new(stat::AtkInterval::new(0.5)),
                    ..default()
                },
                ..default()
            },
            AttackCycle::new(Duration::from_millis(200), Duration::from_millis(150)),
            Melee::default(),
            Range::from_vertices([
                Vec2::new(1.5, -0.5),
                Vec2::new(1.5, 0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(-0.5, -0.5),
            ]),
        ))
        .set_parent(grid)
        .with_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(shape::Cube::new(0.8).into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::CYAN,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.4, 0.0),
                    ..default()
                });

            /*
            parent
                .spawn((
                    SpatialBundle::default(),
                    HpDecay::new(90.0),
                ));*/
        });

    // FIXME: test enemy
    commands
        .spawn((
            EnemyBundle {
                transform: Transform::from_xyz(6.0, 0.0, 0.0),
                follower: Follower::new([
                    Checkpoint::at(Vec2::new(-6.0, 0.0)),
                    Checkpoint::at(Vec2::new(6.0, 3.0)),
                    Checkpoint::at(Vec2::new(-6.0, 3.0)),
                    Checkpoint::at(Vec2::new(-5.0, -3.0)),
                    Checkpoint::at(Vec2::new(6.0, 0.0)),
                    Checkpoint::at(Vec2::new(0.0, 3.0)),
                    Checkpoint::at(Vec2::new(0.0, 0.0)),
                    Checkpoint::at(Vec2::new(6.0, 3.0)),
                ]),
                stats: EnemyStatBundle {
                    hp: StatBundle::new(stat::MaxHp::new(4000.0)),
                    atk: StatBundle::new(stat::Atk::new(420)),
                    def: StatBundle::new(stat::Def::new(80)),
                    res: StatBundle::new(stat::Res::new(0)),
                    ..default()
                },
                hatred: spcc::battle::targeting::Hatred(1),
                ..default()
            },
            AttackCycle::new(Duration::from_millis(200), Duration::from_millis(150)),
            Melee::default(),
            NavBundle::default(),
            spcc::battle::damage::DespawnOnDeath::new(Duration::from_millis(200)),
        ))
        .with_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(shape::UVSphere {
                        radius: 0.25,
                        sectors: 16,
                        stacks: 16,
                    }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::RED,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.125, 0.0),
                    ..default()
                });

            parent
                .spawn((
                    SpatialBundle::default(),
                    stat::MoveSpeed::modif().add(0.5),
                ));

        });
}

pub fn setup(
    mut commands: Commands,
    mut stage_load_tx: EventWriter<LoadStageEvent>,
    mut gizmo_config: ResMut<GizmoConfig>,
) {
    // DEBUG: setup gizmo config
    gizmo_config.depth_bias = -1.;

    // create camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 8.0).looking_at(Vec3::Z, Vec3::Y),
            ..default()
        },
        //RaycastPickCamera::default(),
    ));

    // begin stage loading
    stage_load_tx.send(StageBuilder::new("maps/ccmap.ron").into());
}
