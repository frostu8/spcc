//! Navigation utilities using grid-based navigation.

use super::*;

use std::cmp::Ordering;
use std::collections::{HashMap, BinaryHeap};

use bevy::prelude::*;

pub struct NavPlugin;

impl Plugin for NavPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update, 
                (
                    compute_navigation,
                    debug_show_navigation,
                )
            );
    }
}

/// An entity that is trying to navigate through an environment.
#[derive(Clone, Component, Debug, Default)]
pub struct Nav {
    target: Vec3,
    path: Vec<Coordinates>,
}

impl Nav {
    /// Creates a new `Nav`.
    pub fn new(target: Vec3) -> Nav {
        Nav {
            target,
            path: Vec::default(),
        }
    }

    /// The target of the nav.
    pub fn target(&self) -> Vec3 {
        self.target
    }

    /// Sets the target of the nav.
    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
    }
}

/// A pathfinder for a [`Grid`].
pub struct Pathfinder<'a> {
    grid: &'a Grid,
}

impl<'a> Pathfinder<'a> {
    /// Creates a new `Pathfinder`.
    pub fn new(grid: &'a Grid) -> Pathfinder<'a> {
        Pathfinder {
            grid,
        }
    }

    /// Finds a path between two [`TileKind::Ground`][1] tiles using the A*
    /// algorithm.
    ///
    /// Assumes the starting node is a valid node.
    ///
    /// [1]: crate::tile_map::TileKind
    pub fn find_path(&self, start: Coordinates, end: Coordinates) -> Result<Vec<Coordinates>, NoPathError> {
        let start: IVec2 = start.into();
        let end: IVec2 = end.into();

        let mut open = BinaryHeap::<GridNode>::new();
        let mut memory = HashMap::<IVec2, IVec2>::new();

        // initialize with starting node
        open.push(GridNode {
            pos: start,
            distance_squared: start.distance_squared(end),
        });

        while let Some(current) = open.pop() {
            if current.pos == end {
                // end found!!! reconstruct path
                let mut path = vec![current.pos];

                while let Some(next) = memory.get(&path[path.len() - 1]) {
                    path.push(*next);
                }

                return Ok(path
                    .into_iter()
                    .rev()
                    .map(|s| s.into())
                    .collect());
            }

            // get neighbors
            for neighbor in [IVec2::X, IVec2::Y, -IVec2::X, -IVec2::Y] {
                let neighbor = current.pos + neighbor;

                // do not visit start tile
                if neighbor == start {
                    continue;
                }

                // check if we haven't already visited this
                if memory.contains_key(&neighbor) {
                    continue;
                }

                // check if neighbor is valid
                let tile = self.grid.get(&neighbor.into());

                if let Some(tile) = tile {
                    if !tile.is_solid() {
                        // add neighbor to open list
                        open.push(GridNode {
                            pos: neighbor,
                            distance_squared: neighbor.distance_squared(end),
                        });

                        // also add neighbor to memory so we can backtrack
                        // later
                        memory.insert(neighbor, current.pos);
                    }
                }
            }
        }

        Err(NoPathError)
    }

    /// Preforms a line-of-sight (LOS) check between two arbitrary local
    /// positions.
    ///
    /// Returns the tile that made this test fail, or `None` if the test was
    /// successful.
    pub fn los_check(&self, _start: Vec3, _end: Vec3) -> Option<Coordinates> {
        // http://playtechs.blogspot.com/2007/03/raytracing-on-grid.html?m=1
        todo!()
    }
}

/// No valid path was found.
#[derive(Debug)]
pub struct NoPathError;

/// Grid node for use in [`Pathfinder::find_path`].
///
/// `GridNode`s are ordered in descending distance.
#[derive(PartialEq, Eq)]
struct GridNode {
    pos: IVec2,
    distance_squared: i32,
}

impl PartialOrd for GridNode {
    fn partial_cmp(&self, other: &GridNode) -> Option<Ordering> {
        self
            .distance_squared
            .partial_cmp(&other.distance_squared)
            .map(|o| o.reverse())
    }
}

impl Ord for GridNode {
    fn cmp(&self, other: &GridNode) -> Ordering {
        self
            .distance_squared
            .cmp(&other.distance_squared)
            .reverse()
    }
}

pub fn compute_navigation(
    mut query: Query<(&GlobalTransform, &mut Nav)>,
    grid_query: Query<(&Grid, &GlobalTransform)>,
    //tile_query: Query<(&Tile, &Transform)>,
) {
    let Ok((grid, grid_transform)) = grid_query.get_single() else {
        return;
    };

    for (global_transform, mut nav) in query.iter_mut() {
        // TODO: do checking to see if a path needs to be rebuilt
        // for now this only happens once
        if nav.path.len() > 0 {
            continue;
        }

        let pathfinder = Pathfinder::new(grid);

        // do grid-based a* pathfinding
        // convert world coordinates to local
        let start = grid_transform.affine().inverse().transform_point(global_transform.translation());
        let target = grid_transform.affine().inverse().transform_point(nav.target);

        // attempt to locate tile this nav is on
        let start = Coordinates::from_local(start);
        let target = Coordinates::from_local(target);

        // pathfind
        if let Ok(path) = pathfinder.find_path(start, target) {
            nav.path = path;
        }
    }
}

pub fn debug_show_navigation(
    query: Query<(&GlobalTransform, &Nav)>,
    grid_query: Query<&GlobalTransform, With<Grid>>,
    mut gizmos: Gizmos
) {
    let Ok(grid_transform) = grid_query.get_single() else {
        return;
    };

    for (transform, nav) in query.iter() {
        // draw path
        for (first, next) in nav.path.iter().zip(nav.path.iter().skip(1)) {
            let start = first.local(0.0);
            let end = next.local(0.0);

            let start = grid_transform.transform_point(start);
            let end = grid_transform.transform_point(end);

            gizmos.line(start, end, Color::CYAN);
        }

        gizmos
            .circle(
                transform.translation(),
                Vec3::Y,
                0.05,
                Color::GREEN,
            );

        gizmos
            .circle(
                nav.target,
                Vec3::Y,
                0.05,
                Color::RED,
            );
    }
}

