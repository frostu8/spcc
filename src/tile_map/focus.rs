//! Focus entities that are grid-locked, using [`bevy_mod_picking`].

use super::*;
use super::range::Range;

use bevy::prelude::*;

use bevy_mod_picking::prelude::*;

pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                (
                    allow_focus_on_tiles,
                    highlight_range_focus,
                )
            );
    }
}

/// A component marking a grid entity that is currently focused.
#[derive(Clone, Component, Debug, Default)]
pub struct Focus;

pub fn allow_focus_on_tiles(
    mut commands: Commands,
    query: Query<Entity, Added<Tile>>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((
                RaycastPickTarget::default(),
                On::<Pointer<Click>>::run(change_focus),
            ));
    }
}

pub fn highlight_range_focus(
    query: Query<(Entity, &Range, &Coordinates), Added<Focus>>,
    mut tile_query: Query<&mut Handle<TileHighlightMaterial>>,
    parents_query: Query<&Parent>,
    grid_query: Query<&Grid>,
    grid_assets: Res<GridAssets>,
) {
    for (entity, range, coordinates) in query.iter() {
        // highlight all tiles in range
        for parent in parents_query.iter_ancestors(entity) {
            let Ok(grid) = grid_query.get(parent) else {
                continue;
            };

            // access tiles
            for tile in range.tiles() {
                let final_tile = *coordinates + *tile;

                let Some(tile) = grid.lookup.get(&final_tile) else {
                    continue;
                };

                if let Ok(mut material) = tile_query.get_mut(tile.entity) {
                    // TODO: colors
                    *material = grid_assets.hostile_indicator.clone();
                }
            }
        }
    }
}

pub fn change_focus(
    mut commands: Commands,
    listener: Listener<Pointer<Click>>,
    grid_query: Query<Entity, With<Grid>>,
    tile_coordinates_query: Query<(&Coordinates, &Parent), With<Tile>>,
    coordinates_query: Query<(Entity, &Coordinates, &Parent), Without<Tile>>,
) {
    let tile = listener.target();

    // make sure our actions are constrained within the grid
    let (tile_coords, parent) = tile_coordinates_query.get(tile).unwrap();
    let grid = grid_query.get(parent.get()).unwrap();

    // find entity with coordinates under parent that isn't tile
    for (entity, coords, parent) in coordinates_query.iter() {
        if parent.get() == grid && coords == tile_coords {
            commands
                .entity(entity)
                .insert(Focus);
        }
    }
}
