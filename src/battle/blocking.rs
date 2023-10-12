//! Blocking stfuf (I hate programming) :(

use bevy::prelude::*;

use crate::tile_map::nav::{Nav, NavSystem};
use crate::stats::{stat, ComputedStat};

use super::damage::DeathEvent;
use super::auto_attack::AttackCycle;
use super::BoundingCircle;

pub struct BlockingPlugin;

impl Plugin for BlockingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    start_blocking
                        .before(NavSystem::Steering),
                    disable_nav_for_blocking
                        .before(NavSystem::Steering)
                        .after(start_blocking),
                    disengage_dead_blockers
                        .before(disable_nav_for_blocking)
                        .after(start_blocking),
                    reset_attack_cycles
                        .after(start_blocking),
                )
            );
    }
}

/// An entity that can block other entities.
///
/// How much the entity can block is detemrined by the [`stat::Block`].
#[derive(Clone, Component, Debug)]
pub struct Blocker {
    pub can_block: bool,
    pub blocking: Vec<Entity>,
}

impl Default for Blocker {
    fn default() -> Blocker {
        Blocker {
            can_block: true,
            blocking: Vec::new(),
        }
    }
}

/// An entity that can be blocked by another entity.
///
/// If the entity has an `AttackCycle` component, changes to this will reset
/// the `AttackCycle`.
#[derive(Clone, Component, Debug, Default)]
pub struct Blockable {
    pub blocked_by: Option<Entity>,
}

impl Blockable {
    pub fn is_blocked(&self) -> bool {
        self.blocked_by.is_some()
    }
}

pub fn reset_attack_cycles(
    mut blockable_query: Query<&mut AttackCycle, Changed<Blockable>>,
) {
    for mut attack_cycle in blockable_query.iter_mut() {
        attack_cycle.reset();
    }
}

pub fn disengage_dead_blockers(
    mut blocker_query: Query<&mut Blocker>,
    mut blockable_query: Query<&mut Blockable>,
    mut death_event_rx: EventReader<DeathEvent>,
) {
    for death_event in death_event_rx.iter() {
        let Ok(mut blocker) = blocker_query.get_mut(death_event.0) else {
            continue;
        };

        // clear blockable status
        for entity in blocker.blocking.iter() {
            let Ok(mut blockable) = blockable_query.get_mut(*entity) else {
                continue;
            };

            blockable.blocked_by = None;
        }

        // clear blocker status
        blocker.blocking.clear();
        blocker.can_block = false;
    }
}

pub fn disable_nav_for_blocking(
    mut query: Query<(&Blockable, &mut Nav), Changed<Blockable>>,
) {
    for (blockable, mut nav) in query.iter_mut() {
        nav.active = !blockable.is_blocked();
    }
}

pub fn start_blocking(
    mut blockable_query: Query<(Entity, &GlobalTransform, &BoundingCircle, &mut Blockable)>,
    mut blocker_query: Query<(Entity, &GlobalTransform, &BoundingCircle, &mut Blocker, &ComputedStat<stat::Block>)>,
) {
    for (
        blockable_entity,
        blockable_transform,
        blockable_bounding_circle,
        mut blockable,
    ) in blockable_query.iter_mut() {
        // no need to start another blocking interaction if this is already
        // being blocked.
        if blockable.is_blocked() {
            continue;
        }

        // project to 2D XZ plane
        let pos = Vec2::new(
            blockable_transform.translation().x,
            blockable_transform.translation().z,
        );

        for (
            blocker_entity,
            blocker_transform,
            blocker_bounding_circle,
            mut blocker,
            block_stat,
        ) in blocker_query.iter_mut() {
            if !blocker.can_block {
                continue;
            }

            // project to 2D XZ plane
            let blocker_pos = Vec2::new(
                blocker_transform.translation().x,
                blocker_transform.translation().z,
            );

            // compare distances
            let min_distance = blockable_bounding_circle.radius + blocker_bounding_circle.radius;

            let distance = pos.distance(blocker_pos);

            if distance <= min_distance {
                // this blocker is ready to block this blockable!
                // oh god wtf is this sentence

                // make sure the entity can actually block more
                if (blocker.blocking.len() as i32) < block_stat.get() {
                    // setup blocking pointers
                    blocker.blocking.push(blockable_entity);
                    blockable.blocked_by = Some(blocker_entity);
                }
            }
        }
    }
}

