//! Damage helpers and types.

use bevy::prelude::*;

use std::iter::once;
use std::time::Duration;

use crate::stats::{stat, ComputedStat};

/// Plugin for damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DeathEvent>()
            .add_systems(
                Update,
                tick_attack_cycle_timers,
            )
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

/// Basic attack cycling.
///
/// Functions very similarly to skills, where a completion of an attack cycle
/// will start a chain of events. Unlike skills, the recovery rate is directly
/// determined by the parent's [`stat::AtkInterval`] and [`stat::Aspd`],
/// instead of the skill's own internal recovery rate.
///
/// This only determines the interval that melee attacks will connect and
/// ranged attacks will shoot projectiles (the time the attack connects is
/// subject to **projectile travel time**).
///
/// # Anatomy
/// ```text
///       |                       interval                     |
/// ...---|---frontswing---|---backswing---|-------------------|---...
/// ```
#[derive(Clone, Component, Debug, Default)]
pub struct AttackCycle {
    timer: Timer,
    standby: bool,
    last_elapsed: Duration,

    scaled_frontswing: Duration,
    scaled_backswing: Duration,

    frontswing: Duration,
    backswing: Duration,
}

impl AttackCycle {
    /// Creates a new `AttackCycle`.
    ///
    /// This takes animation details (`frontswing`, `backswing`) so the attacks
    /// line up with the animations.
    pub fn new(frontswing: Duration, backswing: Duration) -> AttackCycle {
        AttackCycle {
            timer: Timer::new(Duration::MAX, TimerMode::Repeating),
            standby: false,
            last_elapsed: Duration::ZERO,

            scaled_frontswing: frontswing,
            scaled_backswing: backswing,

            frontswing,
            backswing,
        }
    }

    /// Resets the attack cycle.
    pub fn reset(&mut self) {
        self.timer.reset();
    }

    /// Gets the interval of the attack cycle.
    pub fn interval(&self) -> Duration {
        self.timer.duration()
    }

    /// Sets the interval of the attack cycle, scaling the time elapsed on the
    /// cycle.
    pub fn set_interval(&mut self, interval: Duration) {
        if self.timer.duration() == interval {
            return;
        }

        // set stopwatch time by ratio
        // (float) SAFETY: duration can only be positive or zero. Only
        // invariant that can be violated is if both durations are zero, which
        // is checked above.
        let ratio = interval.div_duration_f64(self.timer.duration());
        let elapsed = self.timer.elapsed().mul_f64(ratio);

        self.timer = Timer::new(interval, TimerMode::Repeating);
        self.timer.set_elapsed(elapsed);

        // scale frontswing and backswing if necessary
        self.scale_swing_interval();
    }

    /// Sets the standby status of the `AttackCycle`.
    ///
    /// If the `AttackCycle` is on standby, after the current cycle has
    /// finished, the elapsed time will stay at zero until the `AttackCycle`
    /// leaves standby. This is similar to "SP lockout," and will activate if
    /// the targeting cannot find valid targets.
    pub fn set_standby(&mut self, standby: bool) {
        self.standby = standby;
    }

    /// Gets the frontswing period, which may have been scaled by short attack
    /// intervals.
    pub fn frontswing(&self) -> Duration {
        self.scaled_frontswing
    }

    /// Gets the backswing period, which may have been scaled by short attack
    /// intervals.
    pub fn backswing(&self) -> Duration {
        self.scaled_backswing
    }

    /// The amount elapsed in the current attack cycle.
    pub fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }

    /// Ticks the `AttackCycle`.
    pub fn tick(&mut self, delta: Duration) -> &AttackCycle {
        if self.standby && self.timer.elapsed() == Duration::ZERO {
            // do not tick if timer is on standby
            return self;
        }

        // tick timer
        self.last_elapsed = self.timer.elapsed();
        self.timer.tick(delta);

        if self.standby && self.timer.just_finished() {
            // clamp timer
            self.timer.set_elapsed(Duration::ZERO);
        }

        self
    }

    /// Total cycles finished this tick.
    pub fn times_finished_this_tick(&self) -> u32 {
        self.timer.times_finished_this_tick()
    }

    /// Times that an attach should have connected this tick.
    pub fn attacks_this_tick(&self) -> u32 {
        let attack_finished = (self.last_elapsed < self.scaled_frontswing)
            && (self.timer.elapsed() >= self.scaled_frontswing);

        self.times_finished_this_tick().saturating_sub(1)
            + attack_finished as u32
    }

    fn scale_swing_interval(&mut self) {
        let total = self.frontswing + self.backswing;

        // skip cycles with no swing periods
        if total == Duration::ZERO {
            return;
        }

        if total < self.interval() {
            // no scaling is needed
            self.scaled_frontswing = self.frontswing;
            self.scaled_backswing = self.backswing;
        } else {
            // scaling is needed
            let frontswing_ratio = self.frontswing.div_duration_f64(total);
            let backswing_ratio = self.backswing.div_duration_f64(total);

            self.scaled_frontswing = self.interval().mul_f64(frontswing_ratio);
            self.scaled_backswing = self.interval().mul_f64(backswing_ratio);
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

pub fn tick_attack_cycle_timers(
    mut query: Query<(Entity, &mut AttackCycle)>,
    parents_query: Query<&Parent>,
    aspd_stats_query: Query<(&ComputedStat<stat::AtkInterval>, &ComputedStat<stat::Aspd>)>,
    time: Res<Time>,
) {
    for (entity, mut attack_cycle) in query.iter_mut() {
        // adjust timer based on parent or current entity stats
        for parent in parents_query.iter_ancestors(entity).chain(once(entity)) {
            let Ok((atk_interval, aspd)) = aspd_stats_query.get(parent) else {
                continue;
            };

            // find attack interval
            let atk_interval = if aspd.get() > 0 {
                Duration::from_secs_f32(atk_interval.get() + (100.0 / aspd.get() as f32))
            } else {
                Duration::MAX
            };

            // set interval
            attack_cycle.set_interval(atk_interval);
        }

        attack_cycle.tick(time.delta());
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
    mut query: Query<(&mut Health, &ComputedStat<stat::MaxHp>), Changed<ComputedStat<stat::MaxHp>>>,
) {
    for (mut health, max_hp) in query.iter_mut() {
        // adjust max hp
        health.hp = health.hp * max_hp.get() / health.max_hp;
        health.max_hp = max_hp.get();
    }
}

