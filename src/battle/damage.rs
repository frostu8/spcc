//! Damage helpers and types.

use bevy::prelude::*;

use crate::stats::{stat, ComputedStat};

/// Plugin for damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DeathEvent>()
            .add_systems(
                PostUpdate,
                (
                    send_death_event,
                    detect_maxhp_changes,
                ),
            );
    }
}

/// Damage type.
///
/// Determines how final damage will be calculated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageType {
    Physical,
    Arts,
    True,
}

/// The actual HP of an entity.
///
/// This cannot below the entity's [`stat::MaxHp`]. When this hits or goes
/// below `0`, it will be very bad for the entity...
///
/// Or will it? Listen to [`DeathEvent`] for when entities health are depleted.
#[derive(Clone, Component, Debug)]
pub struct Health {
    hp: f32,
    max_hp: f32, // used to track maxhp changes
}

impl Health {
    /// Creates a new `Health` at full HP based on max hp.
    ///
    /// # Panics
    /// Panics if `max_hp` is equal to or less than 0.
    pub fn new(max_hp: f32) -> Health {
        assert!(max_hp > 0.0);

        Health { hp: max_hp, max_hp }
    }

    /// Gets the health points of an entity.
    pub fn get(&self) -> f32 {
        self.hp
    }

    /// Sets the health points of an entity.
    pub fn set(&mut self, hp: f32) {
        self.hp = hp.min(self.max_hp);
    }

    /// Gets the healths points of an entity as a percentage.
    pub fn percentage(&self) -> f32 {
        self.hp as f32 / self.max_hp as f32
    }
}

impl Default for Health {
    fn default() -> Health {
        // This default is mostly so as to not violate an invariant (max_hp <= 0)
        Health::new(1500.0)
    }
}

/// A marker component for dead entities.
///
/// Entities that go below zero hp will be marked with this after the
/// [`send_death_event`] system runs. **However**, this component will *not* be
/// removed if the unit is "revived."
#[derive(Clone, Component, Debug, Default)]
pub struct Dead;

/// An event that fires when an entity's HP reaches zero or below zero.
///
/// Logic that prevents death must be done before the [`send_death_event`]
/// system runs.
#[derive(Debug, Event)]
pub struct DeathEvent(pub Entity);

pub fn send_death_event(
    mut commands: Commands,
    query: Query<(Entity, &Health), Without<Dead>>,
    mut death_event_tx: EventWriter<DeathEvent>,
) {
    for (entity, health) in query.iter() {
        if health.hp <= 0.0 {
            death_event_tx.send(DeathEvent(entity));

            commands
                .entity(entity)
                .insert(Dead);
        }
    }
}

pub fn detect_maxhp_changes(
    mut query: Query<(&mut Health, &ComputedStat<stat::MaxHp>), Changed<ComputedStat<stat::MaxHp>>>,
) {
    for (mut health, max_hp) in query.iter_mut() {
        // adjust max hp
        health.hp = health.hp * max_hp.get() / health.max_hp;
        health.max_hp = max_hp.get();
    }
}

