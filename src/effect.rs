//! Basic status effects.

use crate::battle::damage::Health;

use std::iter::once;
use std::time::Duration;

use bevy::prelude::*;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                tick_hp_decay,
            );
    }
}

/// HP Decay per second.
///
/// Applies to the entity or any parent entity with a [`Health`] component.
#[derive(Clone, Component, Debug)]
pub struct HpDecay {
    hp: f32,
    timer: Timer,
}

impl HpDecay {
    /// Creates a new `HpDecay`.
    pub fn new(hp: f32) -> HpDecay {
        HpDecay {
            hp,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

pub fn tick_hp_decay(
    mut decay_query: Query<(Entity, &mut HpDecay)>,
    parents_query: Query<&Parent>,
    mut health_query: Query<&mut Health>,
    time: Res<Time>,
) {
    for (entity, mut hp_decay) in decay_query.iter_mut() {
        hp_decay.timer.tick(time.delta());
        let ticks = hp_decay.timer.times_finished_this_tick();

        if ticks == 0 {
            continue;
        }

        // find parent
        for entity in once(entity).chain(parents_query.iter_ancestors(entity)) {
            if let Ok(mut health) = health_query.get_mut(entity) {
                let current_hp = health.get();

                // tick hp
                for _ in 0..ticks {
                    health.set(current_hp - hp_decay.hp);
                }
            }
        }
    }
}

