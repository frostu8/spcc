#![feature(div_duration)]

pub mod battle;
pub mod tile_map;
pub mod material;
pub mod stage;
pub mod stats;
pub mod status;
pub mod ui;

use bevy::prelude::*;
use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};

use std::iter::once;

/// The game's state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Splash,
    StageLoading,
    InGame,
}

/// Finds a parent that satisfies a specific read-only query.
pub fn find_parent<'a, T, F>(
    entity: Entity,
    parents_query: &Query<&Parent>,
    query: &'a Query<T, F>,
) -> Option<<T::ReadOnly as WorldQuery>::Item<'a>>
where
    T: WorldQuery,
    F: ReadOnlyWorldQuery,
{
    // look at how nice and straightforward this is
    for parent in once(entity).chain(parents_query.iter_ancestors(entity)) {
        if let Ok(t) = query.get(parent) {
            return Some(t);
        }
    }

    None
}

/// Finds a parent that satisfies a specific query.
pub fn find_parent_mut<'a, T, F>(
    entity: Entity,
    parents_query: &Query<&Parent>,
    query: &'a mut Query<T, F>,
) -> Option<<T as WorldQuery>::Item<'a>>
where
    T: WorldQuery,
    F: ReadOnlyWorldQuery,
{
    // we have to do this weirdness because reborrow rules (what the scallop?)
    // man I love Rust man I love Rust man I love Rust man I love Rust
    // man I love Rust man I love Rust man I love Rust man I love Rust
    // man I love Rust man I love Rust man I love Rust man I love Rust
    // man I love Rust man I love Rust man I love Rust man I love Rust
    let mut result = Option::<Entity>::None;

    for parent in once(entity).chain(parents_query.iter_ancestors(entity)) {
        if query.contains(parent) {
            result = Some(parent);
            break;
        }
    }

    result.and_then(|e| query.get_mut(e).ok())
}

