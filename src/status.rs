//! Basic status effects.

use crate::battle::damage::{DamageType, DamageReceivedEvent, Health};
use crate::stats::{Modifier, stat, Stat};
use crate::find_parent;

use std::time::Duration;

use bevy::prelude::*;

/// Implements basic components that modify parent entities in unique ways.
pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                tick_hp_decay,
            );
    }
}

/// The activated originium buff.
///
/// Can be added to an entity as a bundle, but you should add this bundle as
/// the child of an entity that this buff can act on. See module level details
/// for more information.
#[derive(Bundle, Clone)]
pub struct ActivatedOriginiumStatus {
    atk_buff: <stat::Atk as Stat>::Modifier,
    aspd_buff: <stat::Aspd as Stat>::Modifier,
    hp_decay: HpDecay,
}

impl Default for ActivatedOriginiumStatus {
    fn default() -> ActivatedOriginiumStatus {
        ActivatedOriginiumStatus {
            atk_buff: <stat::Atk as Stat>::Modifier::identity().add(600),
            aspd_buff: <stat::Aspd as Stat>::Modifier::identity().add(50),
            hp_decay: HpDecay::new(150.0),
        }
    }
}

/// HP Decay per interval.
///
/// Applies to the entity or any parent entity with a [`Health`] component.
#[derive(Clone, Component, Debug)]
pub struct HpDecay {
    hp: f32,
    timer: Timer,
}

impl HpDecay {
    /// Creates a new `HpDecay` that decreases the parent entity's health by
    /// `hp` every second.
    pub fn new(hp: f32) -> HpDecay {
        HpDecay {
            hp,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }

    /// Changes the interval of the `HpDecay`.
    pub fn with_interval(self, interval: Duration) -> HpDecay {
        HpDecay {
            timer: Timer::new(interval, TimerMode::Repeating),
            ..self
        }
    }
}

pub fn tick_hp_decay(
    mut decay_query: Query<(Entity, &mut HpDecay)>,
    parent_query: Query<&Parent>,
    health_query: Query<Entity, With<Health>>,
    mut damage_received_tx: EventWriter<DamageReceivedEvent>,
    time: Res<Time>,
) {
    for (entity, mut hp_decay) in decay_query.iter_mut() {
        hp_decay.timer.tick(time.delta());
        let ticks = hp_decay.timer.times_finished_this_tick();

        if ticks > 0 {
            // find parent
            if let Some(entity) = find_parent(
                entity, 
                &parent_query,
                &health_query,
            ) {
                // tick hp
                damage_received_tx.send(DamageReceivedEvent::new(entity)
                    .with_type(DamageType::True)
                    .with_damage(hp_decay.hp * ticks as f32));
            }
        }
    }
}

