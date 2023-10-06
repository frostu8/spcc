//! Navigation utilities using grid-based navigation.

use super::*;

use crate::stats::stat;

use bevy::prelude::*;

pub struct NavPlugin;

impl Plugin for NavPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update, 
                (
                    compute_navigation,
                )
            );
    }
}

/// An entity that is trying to navigate through an environment.
#[derive(Clone, Component, Debug, Default)]
pub struct Nav {
    target: Vec3,
    cached_waypoints: Vec<Vec2>,
    waypoint_idx: usize,
}

impl Nav {
    /// Creates a new `Nav`.
    pub fn new(target: Vec3) -> Nav {
        Nav {
            target,
            cached_waypoints: Default::default(),
            waypoint_idx: 0,
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

    fn calculate_nav(
        &mut self,
        my_transform: &Transform,
        tile_query: &Query<(&Tile, &Transform)>,
        grid: &Grid,
        grid_transform: &GlobalTransform,
    ) {
        let pathfinder = Pathfinder::new(grid);

        let start = grid_transform.transform_point(self.target);
        let target = grid_transform.transform_point(self.target);

        // attempt to locate tile this nav is on
        let start = Coordinates::from_local(start);
        let target = Coordinates::from_local(target);

        // pathfind
        let path = pathfinder.find_path(start, target);
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

    /// Finds a path between two tiles using the A* algorithm.
    pub fn find_path(&self, start: Coordinates, end: Coordinates) -> Vec<Coordinates> {
        todo!()
    }

    /// Preforms a line-of-sight (LOS) check between two arbitrary local
    /// positions.
    ///
    /// Returns the tile that made this test fail, or `None` if the test was
    /// successful.
    pub fn los_check(&self, start: Vec3, end: Vec3) -> Option<Coordinates> {
        // http://playtechs.blogspot.com/2007/03/raytracing-on-grid.html?m=1
        todo!()
    }
}

pub fn compute_navigation(
    mut query: Query<(&mut Transform, &Nav, &stat::MoveSpeed)>,
    tile_query: Query<(&Tile, &Transform)>,
    grid_query: Query<(&Grid, &GlobalTransform)>,
) {
    let (grid, grid_transform) = grid_query.single();

    for (mut transform, nav, move_speed) in query.iter_mut() {
    }
}

