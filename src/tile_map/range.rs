//! Utilities involving grid ranges.

use bevy::prelude::*;

use super::Coordinates;

/// A range.
#[derive(Clone, Component, Debug, Default)]
pub struct Range {
    tiles: Vec<Coordinates>,
    direction: Direction,
}

impl Range {
    /// Creates a new `Range`.
    ///
    /// The default [`Direction`] is [`Direction::Right`].
    pub fn new(tiles: impl Into<Vec<Coordinates>>) -> Range {
        Range {
            tiles: tiles.into(),
            direction: Direction::Right,
        }
    }

    /// The tiles of the `Range`.
    pub fn tiles(&self) -> &[Coordinates] {
        &self.tiles
    }

    /// The direction the `Range` is facing.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Turns the `Range` to face in a [`Direction`]
    pub fn face_to(&mut self, direction: Direction) {
        let diff = self.direction.difference(direction);

        for tile in self.tiles.iter_mut() {
            tile.x = tile.x * diff.cos() - tile.y * diff.sin();
            tile.y = tile.x * diff.sin() + tile.y * diff.cos();
        }

        self.direction = direction;
    }
}

/// A direction for a range.
#[derive(Clone, Copy, Debug, Default)]
pub enum Direction {
    #[default]
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    /// How many 90-degree turns one direction is from another.
    pub fn difference(self, other: Direction) -> Direction {
        Direction::from_turn_count(self.turn_count() - other.turn_count())
    }

    pub fn sin(self) -> i32 {
        match self {
            Direction::Right => 0,
            Direction::Up => 1,
            Direction::Left => 0,
            Direction::Down => -1,
        }
    }

    pub fn cos(self) -> i32 {
        match self {
            Direction::Right => 1,
            Direction::Up => 0,
            Direction::Left => -1,
            Direction::Down => 0,
        }
    }

    fn turn_count(self) -> i32 {
        match self {
            Direction::Right => 0,
            Direction::Up => 1,
            Direction::Left => 2,
            Direction::Down => 3,
        }
    }

    fn from_turn_count(i: i32) -> Direction {
        match i % 4 {
            0 => Direction::Right,
            1 => Direction::Up,
            2 => Direction::Left,
            3 => Direction::Down,
            _ => unreachable!(),
        }
    }
}

