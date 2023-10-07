//! Entity statistics.
//!
//! These are systems and components that work on [`Stat`]s, that can be
//! queried like any other component using [`ComputedStat`].

pub mod stat;

use std::ops::Deref;

use bevy::prelude::*;

pub struct StatPlugin;

impl Plugin for StatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_stat::<stat::MaxHp>()
            .add_stat::<stat::Atk>()
            .add_stat::<stat::Def>()
            .add_stat::<stat::Res>()
            .add_stat::<stat::AtkInterval>()
            .add_stat::<stat::MoveSpeed>()
            .add_stat::<stat::RedeployTime>()
            .add_stat::<stat::DpCost>()
            .add_stat::<stat::Block>();
    }
}

/// A bundle for enemy stats.
#[derive(Clone, Debug, Default, Bundle)]
pub struct EnemyStatBundle {
    hp: StatBundle<stat::MaxHp>,
    atk: StatBundle<stat::Atk>,
    def: StatBundle<stat::Def>,
    res: StatBundle<stat::Res>,
    atk_interval: StatBundle<stat::AtkInterval>,
    aspd: StatBundle<stat::Aspd>,
    move_speed: StatBundle<stat::MoveSpeed>,
}

/// A bundle used to give an entity a single stat.
#[derive(Clone, Debug, Default, Bundle)]
pub struct StatBundle<T>
where
    T: Stat,
{
    /// The base stat an operator or entity is given.
    pub stat: T,
    /// Computed stat. This is what should be queried for skills and such.
    pub computed_stat: ComputedStat<T>,
}

impl<T> StatBundle<T>
where
    T: Stat,
{
    /// Creates a new stat bundle with a base stat.
    pub fn new(base_stat: T) -> StatBundle<T> {
        StatBundle {
            computed_stat: ComputedStat(base_stat.clone()),
            stat: base_stat,
        }
    }
}

/// Labels for systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum StatSystem {
    PropagateStats,
}

/// A computed final stat.
#[derive(Clone, Debug, Default, Component)]
pub struct ComputedStat<T>(T);

impl<T> Deref for ComputedStat<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The base trait for all supported stats.
pub trait Stat: Component + Clone + PartialEq {
    type Modifier: Modifier;

    /// Determines the logic of applying modifiers to stats.
    fn apply(&mut self, modif: &Self::Modifier);

    /// Creates a modifier that, when combined with another, does nothing.
    ///
    /// Convenience function.
    fn modif() -> Self::Modifier {
        Self::Modifier::identity()
    }
}

/// The base trait for modifiers.
///
/// If you add the [`Stat::Modifier`] as a component to an entity whose parent
/// has the [`Stat`] component, the stats will propagate upwards.
pub trait Modifier: Component {
    /// Creates a modifier that, when combined with another, does nothing.
    fn identity() -> Self;

    /// Creates a modifier that, when applied to a stat, does nothing to it.
    fn base() -> Self;

    /// Combines two other modifiers together.
    fn combine(&mut self, other: &Self);
}

#[derive(Debug, Clone, Component)]
pub struct ModifierI32<T: Send + Sync + 'static> {
    add: i32,
    mul: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> ModifierI32<T> {
    /// How much will be added to the final stat.
    pub fn add(self, value: i32) -> ModifierI32<T> {
        ModifierI32 {
            add: value,
            ..self
        }
    }

    /// How much will be multiplied to the final stat.
    pub fn mul(self, value: f32) -> ModifierI32<T> {
        ModifierI32 {
            mul: value,
            ..self
        }
    }
}

impl<T: Send + Sync + 'static> Modifier for ModifierI32<T> {
    fn identity() -> ModifierI32<T> {
        ModifierI32 {
            add: 0,
            mul: 0.0,
            _marker: std::marker::PhantomData,
        }
    }

    fn base() -> ModifierI32<T> {
        ModifierI32 {
            add: 0,
            mul: 1.0,
            _marker: std::marker::PhantomData,
        }
    }

    fn combine(&mut self, other: &ModifierI32<T>) {
        self.add += other.add;
        self.mul += other.mul;
    }
}

#[derive(Debug, Clone, Component)]
pub struct ModifierF32<T> {
    add: f32,
    mul: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> ModifierF32<T> {
    /// How much will be added to the final stat.
    pub fn add(self, value: f32) -> ModifierF32<T> {
        ModifierF32 {
            add: value,
            ..self
        }
    }

    /// How much will be multiplied to the final stat.
    pub fn mul(self, value: f32) -> ModifierF32<T> {
        ModifierF32 {
            mul: value,
            ..self
        }
    }
}

impl<T: Send + Sync + 'static> Modifier for ModifierF32<T> {
    fn identity() -> ModifierF32<T> {
        ModifierF32 {
            add: 0.0,
            mul: 0.0,
            _marker: std::marker::PhantomData,
        }
    }

    fn base() -> ModifierF32<T> {
        ModifierF32 {
            add: 0.0,
            mul: 1.0,
            _marker: std::marker::PhantomData,
        }
    }

    fn combine(&mut self, other: &ModifierF32<T>) {
        self.add += other.add;
        self.mul += other.mul;
    }
}

/// Propagates stats.
pub fn propagate_stat<T: Stat>(
    mut query: Query<(Entity, &T, &mut ComputedStat<T>)>,
    children: Query<&Children>,
    modifiers: Query<&T::Modifier>,
) {
    for (entity, base_stat, mut final_stat) in query.iter_mut() {
        // create an empty modifier
        let mut final_mod = T::Modifier::base();

        // accumulate modifiers
        for descendant in children.iter_descendants(entity) {
            if let Ok(modif) = modifiers.get(descendant) {
                final_mod.combine(modif);
            }
        }

        // finalize stat
        let mut result_stat = base_stat.clone();
        result_stat.apply(&final_mod);

        if result_stat != final_stat.0 {
            final_stat.0 = result_stat;
        }
    }
}

/// Extension trait for adding stats to [`App`]s.
pub trait AddStatExt {
    /// Adds the systems required to propogate stat buffs.
    fn add_stat<T>(&mut self) -> &mut Self
    where
        T: Stat;
}

impl AddStatExt for App {
    fn add_stat<T>(&mut self) -> &mut App
    where
        T: Stat
    {
        self
            .add_systems(PostUpdate, propagate_stat::<T>.in_set(StatSystem::PropagateStats));
        
        self
    }
}
