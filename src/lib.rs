pub mod enemy;
pub mod tile_map;
pub mod material;
pub mod stage;
pub mod stats;

use bevy::prelude::States;

/// The game's state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Splash,
    StageLoading,
    InGame,
}
