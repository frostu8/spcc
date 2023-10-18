//! Targeting structs.
//!
//! For an entity to be targetable, it needs a [`BoundingCircle`] component (to
//! determine whether the entity is in range) and a [`Hatred`] component (to
//! sort multiple targets).

mod priority;

pub use priority::{TargetingTree, Hatred};

use std::fmt::{self, Formatter, Debug};
use std::ops::Deref;

use bevy::prelude::*;

use parry2d::shape::{Ball, TriMesh};

use super::{BoundingCircle, Hostility};
use super::blocking::{Blocker, Blockable};

/// Targeting plugin.
pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetingTree>()
            .add_systems(
                Update,
                priority::sort_targets.in_set(TargetingSystems::SortTargets),
            )
            .add_systems(Update,
                (clear_targets, priority_blocked_targets, priority_blocker_target, search_targets)
                    .chain()
                    .in_set(TargetingSystems::SearchTargets)
                    .after(TargetingSystems::SortTargets),
            );
    }
}

/// The targeting system sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum TargetingSystems {
    /// Sorts entities' targeting based on [`Hatred`] and taunt.
    SortTargets,
    /// Actually does the target finding. If you want to use information stored
    /// in [`Targets`], run your system **after** this set.
    SearchTargets,
}

/// The "range" of an entity.
#[derive(Clone, Component)]
pub struct Range {
    shape: Shape,
}

#[derive(Clone)]
enum Shape {
    Polygon(TriMesh),
    Circle(Ball),
}

impl Range {
    /// Creates a new `Range` from a set of vertices.
    pub fn from_vertices(vertices: impl IntoIterator<Item = Vec2>) -> Range {
        let mesh = TriMesh::from_polygon(vertices
                .into_iter()
                .map(|s| s.into())
                .collect())
            .unwrap();

        Range {
            shape: Shape::Polygon(mesh),
        }
    }
}

impl Debug for Range {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str("Range(_)")
    }
}

/// Targeting information.
#[derive(Clone, Component, Debug)]
pub struct Targeting {
    /// The maximum amount of targets this entity can have.
    pub max_targets: usize,
}

impl Default for Targeting {
    fn default() -> Targeting {
        Targeting {
            max_targets: 1,
        }
    }
}

/// The actual targeting being stored.
#[derive(Clone, Component, Debug, Default)]
pub struct Targets(Vec<Entity>);

impl Deref for Targets {
    type Target = [Entity];

    fn deref(&self) -> &[Entity] {
        &self.0
    }
}

/// Component for excluding entities from targeting rules.
#[derive(Clone, Component, Debug)]
pub struct Stealth {
    pub visible: bool,
}

impl Default for Stealth {
    fn default() -> Stealth {
        Stealth {
            visible: true,
        }
    }
}

pub fn clear_targets(
    mut query: Query<&mut Targets>,
) {
    for mut targets in query.iter_mut() {
        targets.0.clear();
    }
}

pub fn priority_blocked_targets(
    mut query: Query<(&Targeting, &mut Targets, &Blocker, Option<&Hostility>)>,
    targets_query: Query<(Entity, Option<&Hostility>)>,
) {
    for (targeting, mut found_targets, blocker, hostility) in query.iter_mut() {
        let hostility = hostility.copied().unwrap_or_default();
        
        // add all blocked targets to the list
        let can_take = targeting.max_targets - found_targets.0.len();

        found_targets.0.extend(blocker
            .blocking
            .iter()
            .copied()
            .filter(|e| {
                // check if this entity even still exists
                // this should be the blocking systems problem but we can do
                // this at no cost.
                let Ok((_exists, other_hostility)) = targets_query.get(*e) else {
                    return false;
                };

                hostility.is_hostile_to(&other_hostility.copied().unwrap_or_default())
            })
            .take(can_take));
    }
}

// this system means an enemy with no range can actually attack
pub fn priority_blocker_target(
    mut query: Query<(&Targeting, &mut Targets, &Blockable, Option<&Hostility>)>,
    targets_query: Query<(Entity, Option<&Hostility>)>,
) {
    for (targeting, mut found_targets, blockable, hostility) in query.iter_mut() {
        // skip if we cannot add any more targets
        if found_targets.0.len() >= targeting.max_targets {
            continue;
        }

        let hostility = hostility.copied().unwrap_or_default();

        if let Some(blocked_by) = blockable.blocked_by {
            // check if this entity even still exists
            // this should be the blocking systems problem but we can do
            // this at no cost.
            let Ok((_exists, other_hostility)) = targets_query.get(blocked_by) else {
                continue;
            };

            if hostility.is_hostile_to(&other_hostility.copied().unwrap_or_default()) {
                found_targets.0.push(blocked_by);
            }
        }
    }
}

pub fn search_targets(
    mut targeting_query: Query<(&GlobalTransform, &Targeting, &mut Targets, &Range, Option<&Hostility>)>,
    targets_query: Query<(Entity, &GlobalTransform, &BoundingCircle, Option<&Hostility>, Option<&Stealth>)>,
    targets_tree: Res<TargetingTree>,
) {
    for (
        transform,
        targeting,
        mut found_targets,
        range,
        hostility,
    ) in targeting_query.iter_mut() {
        let hostility = hostility.copied().unwrap_or_default();

        // find suitable targets
        let possible_targets = targets_tree
            .iter()
            .filter_map(|entity| targets_query.get(entity).ok())
            // filter invisible targets
            .filter(|(_, _, _, _, stealth)| {
                stealth.map(|s| s.visible).unwrap_or_else(|| true)
            })
            // filter targets that we aren't hostile to
            .filter(|(_, _, _, target_hostility, _)| {
                hostility.is_hostile_to(&target_hostility.copied().into())
            })
            // filter shapes we intersect with
            .filter(|(_, target_transform, target_bounding_circle, _, _)| {
                match &range.shape {
                    Shape::Polygon(mesh) => {
                        parry2d::query::intersection_test(
                            &global_transform_to_isometry(transform),
                            mesh,
                            &global_transform_to_isometry(target_transform),
                            &target_bounding_circle.0,
                        )
                            .unwrap()
                    }
                    Shape::Circle(ball) => {
                        parry2d::query::intersection_test(
                            &global_transform_to_isometry(transform),
                            ball,
                            &global_transform_to_isometry(target_transform),
                            &target_bounding_circle.0,
                        )
                            .unwrap()
                    }
                }
            });

        let targets = possible_targets
            .map(|(e, _, _, _, _)| e)
            .take(targeting.max_targets);

        found_targets.0 = targets.collect();
    }
}

fn global_transform_to_isometry(t: &GlobalTransform) -> parry2d::math::Isometry<f32> {
    // TODO: rotation support? oh god
    //let (rot, _, _) = t.rotation().to_euler(EulerRot::YXZ);

    parry2d::math::Isometry {
        rotation: default(),
        translation: Vec2::new(t.translation().x, t.translation().z).into(),
    }
}

pub fn debug_draw_targeting(
    query: Query<(&GlobalTransform, &Targets)>,
    position_query: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for (transform, targets) in query.iter() {
        for target in targets.iter() {
            let Ok(target_transform) = position_query.get(*target) else {
                continue;
            };

            gizmos
                .line(
                    transform.translation(),
                    target_transform.translation(),
                    Color::WHITE,
                );
        }
    }
}

pub fn debug_draw_range(
    query: Query<(&GlobalTransform, &Range, Option<&Hostility>)>,
    mut gizmos: Gizmos,
) {
    for (transform, range, hostility) in query.iter() {
        let hostility = hostility.copied().unwrap_or_default();

        let color = match hostility {
            Hostility::Neutral => Color::ORANGE,
            Hostility::Hostile => Color::RED,
            Hostility::Friendly => Color::CYAN,
        };

        match &range.shape {
            Shape::Polygon(mesh) => {
                // draw perimeter of mesh
                for triangle in mesh.triangles() {
                    let a = Vec3::new(triangle.a.x, 0.0, triangle.a.y);
                    let b = Vec3::new(triangle.b.x, 0.0, triangle.b.y);
                    let c = Vec3::new(triangle.c.x, 0.0, triangle.c.y);

                    let a = transform.transform_point(a);
                    let b = transform.transform_point(b);
                    let c = transform.transform_point(c);

                    gizmos.line(a, b, color);
                    gizmos.line(b, c, color);
                    gizmos.line(c, a, color);
                }
            }
            Shape::Circle(ball) => {
                gizmos
                    .circle(
                        transform.translation(),
                        Vec3::Y,
                        ball.radius,
                        color,
                    );
            }
        }
    }
}
