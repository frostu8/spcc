//! Most commonly used, shared stats.

use bevy::prelude::*;

macro_rules! impl_stat_i32 {
    ($name:ty, min: $min:literal, max: $max:literal) => {
        impl $name {
            /// Creates a new stat.
            pub fn new(value: i32) -> $name {
                Self(value.clamp($min, $max))
            }

            /// Gets the stat.
            pub fn get(&self) -> i32 {
                self.0
            }

            /// Sets the stat.
            pub fn set(&mut self, value: i32) {
                self.0 = value.clamp($min, $max);
            }
        }

        impl crate::stats::Stat for $name {
            type Modifier = crate::stats::ModifierI32<$name>;

            fn apply(&mut self, modif: &Self::Modifier) {
                // multiply then add
                let res = (self.0 as f32 * modif.mul) as i32 + modif.add;

                // clamp
                self.0 = res.clamp($min, $max);
            }
        }
    };
    ($name:ty, min: $min:literal) => {
        impl $name {
            /// Creates a new stat.
            pub fn new(value: i32) -> $name {
                Self(value.max($min))
            }

            /// Gets the stat.
            pub fn get(&self) -> i32 {
                self.0
            }

            /// Sets the stat.
            pub fn set(&mut self, value: i32) {
                self.0 = value.max($min);
            }
        }

        impl crate::stats::Stat for $name {
            type Modifier = crate::stats::ModifierI32<$name>;

            fn apply(&mut self, modif: &Self::Modifier) {
                // multiply then add
                let res = (self.0 as f32 * modif.mul) as i32 + modif.add;

                // clamp
                self.0 = res.max($min);
            }
        }
    }
}

macro_rules! impl_stat_f32 {
    ($name:ty, min: $min:literal) => {
        impl $name {
            /// Creates a new stat.
            pub fn new(value: f32) -> $name {
                Self(value.max($min))
            }

            /// Gets the stat.
            pub fn get(&self) -> f32 {
                self.0
            }

            /// Sets the stat.
            pub fn set(&mut self, value: f32) {
                self.0 = value.max($min);
            }
        }

        impl crate::stats::Stat for $name {
            type Modifier = crate::stats::ModifierF32<$name>;

            fn apply(&mut self, modif: &Self::Modifier) {
                // multiply then add
                let res = self.0 * modif.mul + modif.add;

                // clamp
                self.0 = res.max($min);
            }
        }
    }
}

/// The maximum HP of an entity.
#[derive(Clone, Component, Debug)]
pub struct MaxHp(i32);

impl Default for MaxHp {
    fn default() -> MaxHp {
        MaxHp(1500)
    }
}

/// The ATK of an entity. Auto-attacks deal 100% ATK as damage.
#[derive(Clone, Component, Debug)]
pub struct Atk(i32);

impl Default for Atk {
    fn default() -> Atk {
        Atk(600)
    }
}

/// The DEF of an entity. Reduces Physical damage taken by a flat amount.
#[derive(Clone, Component, Debug)]
pub struct Def(i32);

impl Default for Def {
    fn default() -> Def {
        Def(600)
    }
}

/// The RES of an entity. Reduces Arts damage taken by a percentage.
///
/// A percentage between 0 and 100. Only whole numbers (for simplicity).
#[derive(Clone, Component, Debug)]
pub struct Res(i32);

impl Default for Res {
    fn default() -> Res {
        Res(0)
    }
}

/// The attack interval of an entity.
///
/// Determines the base speed at which an operator or enemy can schwing in
/// seconds.
#[derive(Clone, Component, Debug)]
pub struct AtkInterval(f32);

impl Default for AtkInterval {
    fn default() -> AtkInterval {
        AtkInterval(1.5)
    }
}

/// Attack speed, an additional modifier to [`AtkInterval`].
///
/// Every 100 ASPD is 1.0x attack speed.
#[derive(Clone, Component, Debug)]
pub struct Aspd(i32);

impl Default for Aspd {
    fn default() -> Aspd {
        Aspd(100)
    }
}

/// **Enemy only** Movement speed in tiles/second.
#[derive(Clone, Component, Debug)]
pub struct MoveSpeed(f32);

impl Default for MoveSpeed {
    fn default() -> MoveSpeed {
        MoveSpeed(0.5)
    }
}

/// **Operator only** Redeployment time in seconds.
///
/// Determines how fast an operator can be redeployed after retreating.
#[derive(Clone, Component, Debug)]
pub struct RedeployTime(f32);

impl Default for RedeployTime {
    fn default() -> RedeployTime {
        RedeployTime(60.0)
    }
}

/// **Operator only** DP cost much DP must be spent to deploy an operator.
#[derive(Clone, Component, Debug)]
pub struct DpCost(i32);

impl Default for DpCost {
    fn default() -> DpCost {
        DpCost(18)
    }
}

/// **Operator only** Block count determines how many enemies an opeator can
/// block.
#[derive(Clone, Component, Debug)]
pub struct Block(i32);

impl Default for Block {
    fn default() -> Block {
        Block(2)
    }
}

impl_stat_i32!(MaxHp, min: 0);
impl_stat_i32!(Atk, min: 0);
impl_stat_i32!(Def, min: 0);
impl_stat_i32!(Res, min: 0, max: 100);
impl_stat_f32!(AtkInterval, min: 0.0);
impl_stat_i32!(Aspd, min: 0);
impl_stat_f32!(MoveSpeed, min: 0.0);
impl_stat_f32!(RedeployTime, min: 0.0);
impl_stat_i32!(DpCost, min: 0);
impl_stat_i32!(Block, min: 0);

