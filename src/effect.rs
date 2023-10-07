//! Basic status effects.

use crate::damage::Health;

use std::iter::once;

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
pub struct HpDecay(pub f32);

pub fn tick_hp_decay(
    decay_query: Query<(Entity, &HpDecay)>,
    parents_query: Query<&Parent>,
    mut health_query: Query<&mut Health>,
    time: Res<Time>,
) {
    for (entity, hp_decay) in decay_query.iter() {
        // find parent
        for entity in once(entity).chain(parents_query.iter_ancestors(entity)) {
            if let Ok(mut health) = health_query.get_mut(entity) {
                let current_hp = health.get();

                health.set(current_hp - hp_decay.0 * time.delta_seconds());
            }
        }
    }
}

