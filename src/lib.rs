pub mod battle;
pub mod damage;
pub mod effect;
pub mod geometry;
pub mod tile_map;
pub mod material;
pub mod stage;
pub mod stats;
pub mod ui;

use bevy::prelude::States;

/// The game's state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Splash,
    StageLoading,
    InGame,
}
