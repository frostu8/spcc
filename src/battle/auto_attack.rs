//! Attack patterns.

use bevy::prelude::*;

use std::time::Duration;

use crate::stats::{find_stats, stat, ComputedStat};

use super::targeting::{Targets, TargetingSystems};
use super::damage::Health;

pub struct AutoAttackPlugin;

impl Plugin for AutoAttackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                tick_attack_cycle_timers,
            )
            .add_systems(
                Update,
                do_melee_auto_attack
                    .after(tick_attack_cycle_timers)
                    .after(TargetingSystems::SearchTargets),
            );
    }
}

/// An autoattack scheme that does damage as soon as the frontswing concludes.
#[derive(Clone, Component, Debug, Default)]
pub struct Melee {
    in_frontswing: bool,
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

        if self.timer.duration() == Duration::ZERO {
            // create new timer
            self.timer = Timer::new(interval, TimerMode::Repeating);
            self.timer.set_elapsed(interval);
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

    /// Checks if the attack cycle is currently in the frontswing.
    pub fn in_frontswing(&self) -> bool {
        self.timer.elapsed() < self.scaled_frontswing
    }

    /// Checks if the attack cycle is currently in the backswing.
    pub fn in_backswing(&self) -> bool {
        self.timer.elapsed() < self.scaled_frontswing + self.scaled_backswing
            && self.timer.elapsed() >= self.scaled_frontswing
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
        self.timer.tick(delta);

        if self.standby && (
            // timer finished
            self.timer.just_finished()
            // or timer hasn't passed past the frontswing
            || self.timer.elapsed() < self.scaled_frontswing
        ) {
            // clamp timer
            self.timer.set_elapsed(Duration::ZERO);
        }

        self
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

pub fn do_melee_auto_attack(
    mut query: Query<(Entity, &AttackCycle, &Targets, &mut Melee)>,
    mut health_query: Query<&mut Health>,
    parents_query: Query<&Parent>,
    atk_stats_query: Query<&ComputedStat<stat::Atk>>,
) {
    for (entity, attack_cycle, targets, mut melee) in query.iter_mut() {
        // check if we can do an attack
        if !attack_cycle.in_frontswing() && melee.in_frontswing {
            let Some(atk) = find_stats(entity, &parents_query, &atk_stats_query) else {
                continue;
            };

            for target in targets.iter() {
                // TODO: do damage reduction shit
                // aka we shouldn't modify health directly like this
                let Ok(mut health) = health_query.get_mut(*target) else {
                    continue;
                };

                let current_hp = health.get();
                health.set(current_hp - atk.get() as f32);
            }
        }

        melee.in_frontswing = attack_cycle.in_frontswing();
    }
}

pub fn tick_attack_cycle_timers(
    mut query: Query<(Entity, &mut AttackCycle)>,
    parents_query: Query<&Parent>,
    aspd_stats_query: Query<(&ComputedStat<stat::AtkInterval>, &ComputedStat<stat::Aspd>)>,
    time: Res<Time>,
) {
    for (entity, mut attack_cycle) in query.iter_mut() {
        // adjust timer based on parent or current entity stats
        let Some((atk_interval, aspd)) = find_stats(entity, &parents_query, &aspd_stats_query) else {
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

        attack_cycle.tick(time.delta());
    }
}
