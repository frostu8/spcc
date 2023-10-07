//! Damage helpers and structs.

use bevy::prelude::*;

use crate::stats::{stat, ComputedStat};

/// Plugin for damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                (
                    detect_maxhp_changes,
                ),
            );
    }
}

/// The actual HP of an entity.
///
/// This cannot below the entity's [`stat::MaxHp`]. When this hits or goes
/// below `0`, it will be very bad for the entity...
///
/// Or will it? Listen to [`HealthDepletedEvent`] for when entities health are
/// depleted.
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

pub fn detect_maxhp_changes(
    mut query: Query<(&mut Health, &ComputedStat<stat::MaxHp>), Changed<ComputedStat<stat::MaxHp>>>,
) {
    for (mut health, max_hp) in query.iter_mut() {
        // adjust max hp
        health.hp = health.hp * max_hp.get() / health.max_hp;
        health.max_hp = max_hp.get();
    }
}

