//! Skills systems and components.
//!
//! A skill is a child of an entity capable of casting the skill. The skill has
//! its own [`Range`] properties.

use bevy::prelude::*;

use std::num::NonZeroU32;
use std::time::Duration;

use crate::battle::targeting::Targets;

pub const BURST_SP_LOCKOUT_DURATION: Duration = Duration::from_millis(750);

/// Plugin for skills.
pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SkillActivationEvent>()
            .add_event::<SkillDeactivationEvent>()
            .add_systems(Update,
                (
                    (
                        deactivate_skills,
                        activate_auto_skills,
                        (take_used_sp, start_lockout_timer),
                    )
                        .chain()
                        .in_set(SkillSystem::ActivateSkill),
                    update_lockout_timer.in_set(SkillSystem::UpdateLockoutTimer),
                    increase_sp_with_time.in_set(SkillSystem::RegenSp),
                ).chain()
            );
    }
}

/// System sets for skills.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum SkillSystem {
    ActivateSkill,
    UpdateLockoutTimer,
    RegenSp,
}

/// A bundle for basic skills.
#[derive(Clone, Debug, Default, Bundle)]
pub struct SkillBundle {
    pub skill: Skill,
    pub skill_lockout_timer: SkillLockoutTimer,
}

/// The base component for any skill.
///
/// This allows the skill to be interacted with.
#[derive(Clone, Component, Debug)]
pub struct Skill {
    sp: f32,
    max_sp: f32,
    sp_lockout: bool,
    overflow: OverflowBehavior,
}

impl Skill {
    /// Creates a new `Skill`, with SP at 0.
    ///
    /// # Panics
    /// Panics if `max_sp` is less than or equal to zero.
    pub fn new(max_sp: f32, overflow: OverflowBehavior) -> Skill {
        assert!(max_sp > 0.0);

        Skill {
            sp: 0.0,
            max_sp,
            sp_lockout: false,
            overflow,
        }
    }

    /// The skill points (SP) of the skill.
    pub fn sp(&self) -> f32 {
        self.sp
    }

    /// The maximum skill points (max SP) of the skill.
    pub fn max_sp(&self) -> f32 {
        self.max_sp
    }

    /// Modifies the [`Skill::sp`] while respecting max SP.
    ///
    /// This function does nothing while the `Skill` is in
    /// [`Skill::sp_lockout`].
    pub fn mutate<F>(&mut self, f: F)
    where
        F: FnOnce(f32) -> f32,
    {
        if !self.sp_lockout {
            let max_sp = match self.overflow {
                OverflowBehavior::Capped => self.max_sp,
                OverflowBehavior::Charge(charges) => self.max_sp * charges.get() as f32,
            };

            self.sp = f(self.sp).clamp(0.0, max_sp);
        }
    }

    /// Percentage of SP to max SP.
    ///
    /// **This number can be greater than 1!** If the `Skill` has an
    /// [`OverflowBehavior`] of [`Charge`][1], the percentage will represent
    /// how many charges have been filled.
    ///
    /// [1]: OverflowBehavior::Charge
    pub fn percentage(&self) -> f32 {
        self.sp / self.max_sp
    }

    /// Whether the skill is in SP lockout (cannot gain or lose SP).
    pub fn sp_lockout(&self) -> bool {
        self.sp_lockout
    }

    /// Sets the SP lockout status.
    ///
    /// This allows SP lockout status to be manually turned on, but for almost
    /// all intents and purposes, [`SkillLockoutTimer`] is better.
    pub fn set_sp_lockout(&mut self, sp_lockout: bool) {
        self.sp_lockout = sp_lockout;
    }

    /// Gives the skill initial sp.
    pub fn with_initial_sp(mut self, initial_sp: f32) -> Skill {
        self.mutate(|_| initial_sp);
        self
    }
}

impl Default for Skill {
    fn default() -> Skill {
        Skill {
            sp: 0.0,
            max_sp: 1.0,
            sp_lockout: false,
            overflow: OverflowBehavior::Capped,
        }
    }
}

/// Behavior when sp is added to a skill.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum OverflowBehavior {
    /// SP caps at max SP.
    #[default]
    Capped,
    /// SP caps at a multiple of max SP.
    Charge(NonZeroU32),
}

/// Automatic skill activation.
///
/// Activates the associated skill as soon as a minimum amount of targets are
/// available.
#[derive(Clone, Copy, Component, Debug, Default)]
pub struct AutoSkillActivation {
    /// The minimum amount of targets needed to activate the skill.
    pub min_targets: usize,
}

impl AutoSkillActivation {
    /// Shorthand for auto skill activation that want one target.
    pub fn one() -> AutoSkillActivation {
        AutoSkillActivation {
            min_targets: 1,
        }
    }
}

/// A timer that denotes how long a skill remains in SP lockout.
/// 
/// This timer will start when the [`SkillActivationEvent`] is fired. It will
/// then finish and send the [`SkillDeactivationEvent`], where SP lockout will
/// be released. This creates a continous logical cycle for duration-based
/// skills, but also exists on burst-based skills at a duration of 0.75s.
///
/// This can also be called manually to force a skill in a lockout timer.
#[derive(Clone, Component, Debug, Default)]
pub struct SkillLockoutTimer(Timer);

impl SkillLockoutTimer {
    /// Sets a new skill lockout for `duration`.
    pub fn set(&mut self, duration: Duration) {
        self.0 = Timer::new(duration, TimerMode::Once);
    }
}

/// The duration of a skill.
///
/// Omit this for burst-based skills.
#[derive(Clone, Component, Debug, Default)]
pub struct SkillDuration(pub Duration);

/// The event that is sent when a skill should trigger its effects.
///
/// The only argument is the skill entity itself.
#[derive(Debug, Clone, Event)]
pub struct SkillActivationEvent(pub Entity);

/// The event that is sent when a skill should finish its effects.
///
/// The only argument is the skill entity itself.
#[derive(Debug, Clone, Event)]
pub struct SkillDeactivationEvent(pub Entity);

/// Attached to skills that passively increase with time, where 1 SP is
/// generated over one second.
#[derive(Clone, Copy, Component, Debug, Default)]
pub struct IncreaseWithTime;

fn deactivate_skills(
    query: Query<(Entity, &SkillLockoutTimer)>,
    mut skill_deactivation_tx: EventWriter<SkillDeactivationEvent>,
) {
    for (entity, lockout_timer) in query.iter() {
        if lockout_timer.0.just_finished() {
            skill_deactivation_tx.send(SkillDeactivationEvent(entity));
        }
    }
}

fn activate_auto_skills(
    query: Query<(Entity, &Skill, &AutoSkillActivation, Option<&Targets>)>,
    mut skill_activation_tx: EventWriter<SkillActivationEvent>,
) {
    for (entity, skill, auto_skill, targets) in query.iter() {
        let targets = targets.map(|t| t.len()).unwrap_or_default();

        if skill.percentage() >= 1.0 && auto_skill.min_targets <= targets {
            // trigger skill by sending event
            skill_activation_tx.send(SkillActivationEvent(entity));
        }
    }
}

fn take_used_sp(
    mut query: Query<&mut Skill>,
    mut skill_activation_rx: EventReader<SkillActivationEvent>,
) {
    for event in skill_activation_rx.iter() {
        if let Ok(mut skill) = query.get_mut(event.0) {
            let used_sp = skill.max_sp();
            skill.mutate(|sp| sp - used_sp);
        }
    }
}

fn start_lockout_timer(
    mut query: Query<(&mut SkillLockoutTimer, Option<&SkillDuration>)>,
    mut skill_activation_rx: EventReader<SkillActivationEvent>,
) {
    for event in skill_activation_rx.iter() {
        if let Ok((mut lockout_timer, duration)) = query.get_mut(event.0) {
            // get duration
            let duration = duration
                .map(|s| s.0)
                .unwrap_or_else(|| BURST_SP_LOCKOUT_DURATION);

            // start new lockout timer
            lockout_timer.set(duration);
        }
    }
}

fn update_lockout_timer(
    mut query: Query<(&mut Skill, &mut SkillLockoutTimer)>,
    time: Res<Time>,
) {
    for (mut skill, mut lockout_timer) in query.iter_mut() {
        lockout_timer.0.tick(time.delta());

        // update lockout status
        skill.sp_lockout = !lockout_timer.0.finished();
    }
}

fn increase_sp_with_time(
    mut query: Query<&mut Skill, With<IncreaseWithTime>>,
    time: Res<Time>,
) {
    for mut skill in query.iter_mut() {
        skill.mutate(|sp| sp + time.delta_seconds());

        println!("sp = {}", skill.sp());
    }
}

