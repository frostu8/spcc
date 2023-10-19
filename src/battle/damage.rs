//! Damage helpers and types.

use bevy::prelude::*;

use std::time::Duration;

use crate::stats::{find_stats, stat, ComputedStat};
use crate::ui::StatusBar;

/// Plugin for damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DeathEvent>()
            .add_event::<DamageReceivedEvent>()
            .add_systems(Update,
                (
                    accumulate_damage
                        .in_set(DamageSystems::AccumulateDamage),
                    despawn_on_death,
                    disable_healthbars_for_dead_entities,
                ),
            )
            .add_systems(PostUpdate,
                (detect_maxhp_changes, send_death_event).chain(),
            );
    }
}

/// Damage systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum DamageSystems {
    /// Gathers all damage received events and calculates a final result.
    AccumulateDamage,
}

/// Damage type.
///
/// Determines how final damage will be calculated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DamageType {
    #[default]
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
    max_hp: i32, // used to track maxhp changes
}

impl Health {
    /// Creates a new `Health` at full HP based on max hp.
    ///
    /// # Panics
    /// Panics if `max_hp` is equal to or less than 0.
    pub fn new(max_hp: i32) -> Health {
        assert!(max_hp > 0);

        Health { hp: max_hp as f32, max_hp }
    }

    /// Gets the health points of an entity.
    pub fn get(&self) -> f32 {
        self.hp
    }

    /// Sets the health points of an entity.
    pub fn set(&mut self, hp: f32) {
        self.hp = hp.min(self.max_hp as f32);
    }

    /// Gets the healths points of an entity as a percentage.
    pub fn percentage(&self) -> f32 {
        self.hp as f32 / self.max_hp as f32
    }
}

impl Default for Health {
    fn default() -> Health {
        // This default is mostly so as to not violate an invariant (max_hp <= 0)
        Health::new(1500)
    }
}

/// Damage received event.
///
/// This is raw damage in its purest form. These events document nearly every
/// instance of damage. Each event is one instance of damage, which will
/// **not** be consolidated when it eventually reaches the entity. These events
/// are not necessarily sent by the damage system, but systems should make an
/// effort ot send these before the damage consolidation system.
///
/// This is just so that individual systems do not have to worry about
/// resistant features (there is a lot) and unifies them under a sinngle
/// confound.
#[derive(Clone, Debug, Event)]
pub struct DamageReceivedEvent {
    pub entity: Entity,
    pub damage_type: DamageType,
    pub damage: f32,
}

impl DamageReceivedEvent {
    /// Creates a new `DamageReceivedEvent`.
    pub fn new(entity: Entity) -> DamageReceivedEvent {
        DamageReceivedEvent {
            entity,
            damage_type: DamageType::Physical,
            damage: 0.0,
        }
    }

    /// Constructs a `DamageReceivedEvent` with a [`DamageType`].
    pub fn with_type(self, damage_type: DamageType) -> DamageReceivedEvent {
        DamageReceivedEvent {
            damage_type,
            ..self
        }
    }

    /// Constructs a `DamageReceivedEvent` with a damage amount.
    pub fn with_damage(self, damage: f32) -> DamageReceivedEvent {
        DamageReceivedEvent {
            damage,
            ..self
        }
    }
}

/// A marker component for entities that will despawn after a set amount of
/// time.
///
/// Typically used for enemies only, as operators will lose persistant buffs if
/// they despawn on death.
#[derive(Clone, Component, Debug, Default)]
pub struct DespawnOnDeath {
    timer: Timer,
}

impl DespawnOnDeath {
    /// Creates a new `DespawnOnDeath`.
    pub fn new(duration: Duration) -> DespawnOnDeath {
        DespawnOnDeath {
            timer: Timer::new(duration, TimerMode::Once),
        }
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

/// Accumulates damage received as [`DamageReceivedEvent`]s.
pub fn accumulate_damage(
    mut damage_event_rx: EventReader<DamageReceivedEvent>,
    mut query: Query<&mut Health>,
    parents_query: Query<&Parent>,
    def_stat_query: Query<&ComputedStat<stat::Def>>,
    res_stat_query: Query<&ComputedStat<stat::Res>>,
) {
    for event in damage_event_rx.iter() {
        let Ok(mut health) = query.get_mut(event.entity) else {
            continue;
        };

        // match damage types
        match event.damage_type {
            DamageType::True => {
                // simply apply the damage
                let current_hp = health.get();
                health.set(current_hp - event.damage);
            }
            DamageType::Physical => {
                // get def
                let def = find_stats(
                    event.entity,
                    &parents_query,
                    &def_stat_query,
                )
                    .map(|s| s.get())
                    .unwrap_or_default();

                // reduce damage
                let reduced = (event.damage - def as f32).max(event.damage * 0.05);

                let current_hp = health.get();
                health.set(current_hp - reduced);
            }
            DamageType::Arts => {
                // get res
                let res = find_stats(
                    event.entity,
                    &parents_query,
                    &res_stat_query,
                )
                    .map(|s| s.get())
                    .unwrap_or_default();

                // reduce damage
                let reduced = (event.damage * (res as f32)).max(event.damage * 0.05);

                let current_hp = health.get();
                health.set(current_hp - reduced);
            }
        }
    }
}

pub fn disable_healthbars_for_dead_entities(
    mut query: Query<(&StatusBar, &mut Style)>,
    now_dead_query: Query<Entity, Added<Dead>>,
) {
    for (status_bar, mut style) in query.iter_mut() {
        if now_dead_query.contains(status_bar.entity()) {
            style.display = Display::None;
        }
    }
}

pub fn despawn_on_death(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnOnDeath), With<Dead>>,
    time: Res<Time>,
) {
    for (entity, mut despawn_on_death) in query.iter_mut() {
        despawn_on_death.timer.tick(time.delta());

        if despawn_on_death.timer.just_finished() {
            commands
                .entity(entity)
                .despawn_recursive();
        }
    }
}

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
    mut query: Query<(Entity, &mut Health)>,
    parents_query: Query<&Parent>,
    max_hp_stat_query: Query<&ComputedStat<stat::MaxHp>>,
) {
    for (entity, mut health) in query.iter_mut() {
        let Some(max_hp) = find_stats(entity, &parents_query, &max_hp_stat_query) else {
            continue;
        };

        if max_hp.get() != health.max_hp {
            // adjust max hp
            health.hp = health.hp * max_hp.get() as f32 / health.max_hp as f32;
            health.max_hp = max_hp.get();
        }
    }
}

