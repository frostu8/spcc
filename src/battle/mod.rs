//! Generic battle details.
//!
//! Only components and systems that are related to the action of battle should
//! be placed here, **not** UI or player assistance structs, nor data loading.

pub mod auto_attack;
pub mod damage;
pub mod blocking;
pub mod path;

use damage::Health;

use crate::stats::{EnemyStatBundle, OperatorStatBundle};
use crate::geometry::BoundingCircle;
use crate::tile_map::Coordinates;

use bevy::prelude::*;

/// Battle plugin.
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
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
#[derive(Clone, Debug, Bundle)]
pub struct EnemyBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stats: EnemyStatBundle,
    pub hostility: Hostility,
    pub bounding_circle: BoundingCircle,
    pub health: Health,
    pub follower: path::Follower,
    pub blockable: blocking::Blockable,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        EnemyBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
            stats: default(),
            hostility: Hostility::Hostile,
            bounding_circle: BoundingCircle::new(0.15),
            health: default(),
            follower: default(),
            blockable: default(),
        }
    }
}

/// Operator bundle.
///
/// This is all that is necessary to get a fully-functioning operator entity,
/// complete with blocking, stats, and gridlocks. **Remember to parent this
/// entity to the grid**, or it will not display properly.
#[derive(Clone, Debug, Bundle)]
pub struct OperatorBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stats: OperatorStatBundle,
    pub hostility: Hostility,
    pub bounding_circle: BoundingCircle,
    pub health: Health,
    pub coordinates: Coordinates,
    pub blocker: blocking::Blocker,
}

impl Default for OperatorBundle {
    fn default() -> OperatorBundle {
        OperatorBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
            stats: default(),
            hostility: Hostility::Friendly,
            bounding_circle: BoundingCircle::new(0.4),
            health: default(),
            coordinates: default(),
            blocker: default(),
        }
    }
}

/// Hostility.
///
/// When attached to an entity, determines whether the entity is hostile or
/// friendly and whether it should be targeted as such.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq, Eq, Hash)]
pub enum Hostility {
    #[default]
    Hostile,
    Friendly,
}

