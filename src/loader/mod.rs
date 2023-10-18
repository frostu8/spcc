//! Types for loading and saving `spcc` objects to disk.
//!
//! Until a bevy editor exists (which it won't by the time I finish this
//! project), here is a ridiculously crude scene loading system. Most of this
//! code is "hack quality" at best.

pub mod map;

use map::Map;

use bevy::prelude::*;

use iyes_progress::prelude::*;

use bevy_common_assets::ron::RonAssetPlugin;

use crate::AppState;

/// Loader plugin.
pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ProgressPlugin::new(AppState::StageLoading)
                .continue_to(AppState::InGame)
                .track_assets())
            .add_event::<LoadStageEvent>()
            .init_resource::<StageAssets>()
            .add_systems(Update, begin_loading_stage)
            .add_systems(Update,
                // all loading systems
                (
                    map::load_map,
                ).run_if(in_state(AppState::StageLoading)),
            )
            .add_plugins(RonAssetPlugin::<Map>::new(&["ron"]));
    }
}

/// A stage builder.
///
/// Does not actually do anything on its own, but can be passed as an argument
/// to a loader.
#[derive(Debug, Default, Event)]
pub struct StageBuilder {
    /// The map as a path.
    map_path: String,
}

impl StageBuilder {
    /// Creates a new stage builder, to the specified map.
    pub fn new(map_path: impl Into<String>) -> StageBuilder {
        StageBuilder {
            map_path: map_path.into(),
        }
    }
}

/// Wad of assets for the current loaded stage.
#[derive(Default, Resource)]
pub struct StageAssets {
    /// The specification of the currently loaded map.
    map: Handle<Map>,
}

/// The event that triggers a stage load.
///
/// This event starts so many cascading systems that it would be too
/// complicated to detail the implementation.
///
/// If this event is sent twice in a single frame, only the first one is
/// processed.
#[derive(Debug, Event)]
pub struct LoadStageEvent(pub StageBuilder);

impl From<StageBuilder> for LoadStageEvent {
    fn from(e: StageBuilder) -> LoadStageEvent {
        LoadStageEvent(e)
    }
}

/// Begins loading the stage by loading first-level dependencies of a
/// [`StageBuilder`]. Any consecutive loading will be done when the associated
/// description file loads.
pub fn begin_loading_stage(
    mut load_stage_rx: EventReader<LoadStageEvent>,
    mut stage_assets: ResMut<StageAssets>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    let Some(LoadStageEvent(stage_builder)) = load_stage_rx.iter().next() else {
        return;
    };

    app_state.set(AppState::StageLoading);

    // start loading the map
    let StageAssets {
        map,
        ..
    } = &mut *stage_assets;

    *map = asset_server.load(&stage_builder.map_path);
    loading.add(&*map);

    // load operators, contingencies:
    // lol
}

