//! Enemy structs.

pub mod path;

pub use path::{Checkpoint, Follower};

use bevy::prelude::*;

/// Enemy plugin.
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, _app: &mut App) {
        // TODO Lol
    }
}

/// Enemy bundle.
///
/// This is all that is necessary to get a fully-functioning enemy entity,
/// complete with pathfinding, stats, and health pools. However, there are
/// more, optional components to consider to make your enemy spicy AF:
/// * **Components to display the entity**  
///   As a child of the enemy entity.
///     * [`PbrBundle`] for 3D.
#[derive(Clone, Debug, Default, Bundle)]
pub struct EnemyBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stats: crate::stats::EnemyStatBundle,
    pub follower: Follower,
}

// TODO: tile to world impl, this is just rough
fn tile_to_xz(tile: (u16, u16)) -> Vec2 {
    Vec2::new(tile.0 as f32, tile.1 as f32)
}
