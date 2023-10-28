//! Generic battle details.
//!
//! Only components and systems that are related to the action of battle should
//! be placed here, **not** UI or player assistance structs, nor data loading.

pub mod auto_attack;
pub mod damage;
pub mod blocking;
pub mod path;
pub mod skill;
pub mod targeting;

use damage::Health;

use targeting::{Targeting, Targets, Stealth, Hatred};

pub use crate::stats::{StatBundle, EnemyStatBundle, OperatorStatBundle};
use crate::tile_map::Coordinates;

use parry2d::shape::Ball;

use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;

use std::ops::Deref;

/// Battle plugins.
pub struct BattlePlugins;

impl PluginGroup for BattlePlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>();

        group
            .add(auto_attack::AutoAttackPlugin)
            .add(damage::DamagePlugin)
            .add(blocking::BlockingPlugin)
            .add(path::PathPlugin)
            .add(targeting::TargetingPlugin)
            .add(skill::SkillPlugin)
    }
}

/// Debug draw plugin.
pub struct DebugDrawPlugin;

impl Plugin for DebugDrawPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                (
                    debug_draw_bounding_circle,
                    targeting::debug_draw_range,
                    targeting::debug_draw_targeting,
                )
                    .after(bevy::transform::TransformSystem::TransformPropagate)
            );
    }
}

fn debug_draw_bounding_circle(
    query: Query<(&GlobalTransform, &BoundingCircle, Option<&Hostility>)>,
    mut gizmos: Gizmos,
) {
    for (transform, bounding_circle, hostility) in query.iter() {
        let hostility = hostility.copied().unwrap_or_default();

        let color = match hostility {
            Hostility::Neutral => Color::ORANGE,
            Hostility::Hostile => Color::RED,
            Hostility::Friendly => Color::CYAN,
        };

        gizmos
            .circle(
                transform.translation(),
                Vec3::Z,
                bounding_circle.radius,
                color,
            );
    }
}

/// Enemy bundle.
///
/// This is all that is necessary to get a fully-functioning enemy entity,
/// complete with pathfinding, stats, and health pools. However, there are
/// more, optional components to consider to make your enemy spicy AF:
/// * **Components to display the entity**  
///   As a child of the enemy entity.
///     * [`PbrBundle`] for 3D.
#[derive(Clone, Debug, Bundle)]
pub struct EnemyBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stats: EnemyStatBundle,
    pub hostility: Hostility,
    pub bounding_circle: BoundingCircle,
    pub health: Health,
    pub follower: path::Follower,
    pub blockable: blocking::Blockable,
    pub targeting: Targeting,
    pub targets: Targets,
    pub stealth: Stealth,
    pub hatred: Hatred,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        EnemyBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
            stats: default(),
            hostility: Hostility::Hostile,
            bounding_circle: BoundingCircle::new(0.15),
            health: default(),
            follower: default(),
            blockable: default(),
            targeting: default(),
            targets: default(),
            stealth: default(),
            hatred: default(),
        }
    }
}

/// Operator bundle.
///
/// This is all that is necessary to get a fully-functioning operator entity,
/// complete with blocking, stats, and gridlocks. **Remember to parent this
/// entity to the grid**, or it will not display properly.
#[derive(Clone, Debug, Bundle)]
pub struct OperatorBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub stats: OperatorStatBundle,
    pub hostility: Hostility,
    pub bounding_circle: BoundingCircle,
    pub health: Health,
    pub coordinates: Coordinates,
    pub blocker: blocking::Blocker,
    pub targeting: Targeting,
    pub targets: Targets,
    pub stealth: Stealth,
    pub hatred: Hatred,
}

impl Default for OperatorBundle {
    fn default() -> OperatorBundle {
        OperatorBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
            stats: default(),
            hostility: Hostility::Friendly,
            bounding_circle: BoundingCircle::new(0.5),
            health: default(),
            coordinates: default(),
            blocker: default(),
            targeting: default(),
            targets: default(),
            stealth: default(),
            hatred: default(),
        }
    }
}

/// Hostility.
///
/// When attached to an entity, determines whether the entity is hostile or
/// friendly and whether it should be targeted as such.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq, Eq, Hash)]
pub enum Hostility {
    /// Targets hostile, friendly and other neutral entities.
    ///
    /// Neutral health bars will be colored as [`Hostility::Hostile`] health
    /// bars.
    #[default]
    Neutral,
    /// Targets friendly units.
    Hostile,
    /// Targets hostile units.
    Friendly,
}

impl Hostility {
    /// Checks if this entity should be hostile to another.
    pub fn is_hostile_to(&self, other: &Hostility) -> bool {
        match (self, other) {
            (Hostility::Neutral, _) => true,
            (_, Hostility::Neutral) => true,
            (Hostility::Hostile, Hostility::Friendly) => true,
            (Hostility::Friendly, Hostility::Hostile) => true,
            _ => false,
        }
    }
}

impl From<Option<Hostility>> for Hostility {
    fn from(e: Option<Hostility>) -> Hostility {
        e.unwrap_or_default()
    }
}

/// A 2D bounding circle.
///
/// Determines collision, and if things are in ranges.
#[derive(Debug, Component, Clone)]
pub struct BoundingCircle(Ball);

impl BoundingCircle {
    pub fn new(radius: f32) -> BoundingCircle {
        BoundingCircle(Ball { radius })
    }
}

impl Deref for BoundingCircle {
    type Target = Ball;

    fn deref(&self) -> &Ball {
        &self.0
    }
}

