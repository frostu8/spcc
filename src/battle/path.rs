//! Pathing systems and components for enemies.
//!
//! This does not actually contain path**finding**. See [`nav`][1] of the
//! [`tile_map`][2] module.
//! 
//! [1]: spcc::tile_map::nav
//! [2]: spcc::tile_map

use bevy::prelude::*;

use crate::tile_map::nav::{Nav, NavigationFinishEvent};

/// Pathing plugin.
pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CheckpointPassedEvent>()
            .add_systems(
                Update,
                (
                    start_followers,
                    update_followers_navigation,
                ),
            );
            //.add_systems(Update, follow_path)
            //.add_systems(PostUpdate, start_followers.before(TransformSystem::TransformPropagate));
    }
}

/// Fires when a [`Follower`] passes a checkpoint.
///
/// If the previous checkpoint has a wait time, this is fired **after**
/// waiting.
///
/// The [`Follower::target`] will be the next checkpoint the entity must reach.
/// If it is `None`, the `Follower` is complete.
#[derive(Clone, Debug, Event)]
pub struct CheckpointPassedEvent(pub Entity);

/// A `Follower` paths itself between two or more checkpoints on the map.
///
/// This component (and related systems) do not actually do any work finding
/// paths between the points, they only wait until an entity has made it to the
/// next checkpoint, pops the checkpoint and sends related events to start
/// another path finding.
#[derive(Clone, Debug, Default, Component)]
pub struct Follower {
    checkpoints: Vec<Checkpoint>,
    current_idx: usize,
}

impl Follower {
    /// Creates a new `Follower` with a set of checkpoints.
    pub fn new(checkpoints: impl Into<Vec<Checkpoint>>) -> Follower {
        Follower {
            checkpoints: checkpoints.into(),
            current_idx: 0,
        }
    }

    /// Shorthand to create a `Follower` that spawns at a single checkpoint and
    /// does not move.
    pub fn start_at(checkpoint: Checkpoint) -> Follower {
        Follower {
            checkpoints: vec![checkpoint],
            current_idx: 0,
        }
    }

    /// Checks if the follower's pathing is complete.
    pub fn is_finished(&self) -> bool {
        self.next().is_none()
    }

    /// Fetches a specific checkpoint.
    pub fn get(&self, idx: usize) -> Option<&Checkpoint> {
        self.checkpoints.get(idx)
    }

    /// Peeks the current checkpoint. Returns `None` if there are no
    /// checkpoints left.
    pub fn next(&self) -> Option<&Checkpoint> {
        self.checkpoints.get(self.current_idx)
    }

    /// Advances to the next checkpoint, returning the next checkpoint.
    ///
    /// Mostly used internally, but this can be manually called to skip
    /// checkpoints.
    pub fn advance(&mut self) -> Option<&Checkpoint> {
        self.current_idx += 1;
        self.next()
    }
}

/// A checkpoint is a single destination in a [`Follower`]'s path.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    /// The position to reach.
    pub pos: Vec2,
    /*
    /// How long the [`Follower`] will wait in seconds until moving to the next
    /// checkpoint.
    pub wait_time: f32,
    */
}

impl Checkpoint {
    /// Shorthand for initializing a zero-wait checkpoint.
    pub fn at(pos: Vec2) -> Checkpoint {
        Checkpoint {
            pos,
        }
    }
}

/// System that starts newly spawned [`Follower`]s.
fn start_followers(
    mut query: Query<(&Follower, &mut Nav), Added<Follower>>,
    //mut check_passed_tx: EventWriter<CheckpointPassedEvent>,
) {
    for (follower, mut nav) in query.iter_mut() {
        // set next checkpoint
        if let Some(next) = follower.next() {
            let target = Vec3::new(next.pos.x, 0.0, next.pos.y);
            nav.set_target(target);
        }
    }
    
}

fn update_followers_navigation(
    mut query: Query<(&mut Follower, &mut Nav)>,
    mut nav_finished_events: EventReader<NavigationFinishEvent>,
) {
    for ev in nav_finished_events.iter() {
        if let Ok((mut follower, mut nav)) = query.get_mut(ev.0) {
            // set next checkpoint
            if let Some(next) = follower.advance() {
                let target = Vec3::new(next.pos.x, 0.0, next.pos.y);
                nav.set_target(target);
            }
        }
    }
}

