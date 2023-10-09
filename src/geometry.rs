//! Geometrical utilities.

use bevy::prelude::*;

use crate::battle::Hostility;

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                PostUpdate,
                debug_draw_bounding_circle,
            );
    }
}

/// A 2D bounding circle.
///
/// Determines collision, and if things are in ranges.
#[derive(Debug, Component, Clone)]
pub struct BoundingCircle {
    pub radius: f32,
}

impl BoundingCircle {
    pub fn new(radius: f32) -> BoundingCircle {
        BoundingCircle { radius }
    }
}

fn debug_draw_bounding_circle(
    query: Query<(&GlobalTransform, &BoundingCircle, Option<&Hostility>)>,
    mut gizmos: Gizmos,
) {
    for (transform, bounding_circle, hostility) in query.iter() {
        let hostility = hostility.copied().unwrap_or_default();

        let color = match hostility {
            Hostility::Hostile => Color::RED,
            Hostility::Friendly => Color::CYAN,
        };

        gizmos
            .circle(
                transform.translation(),
                Vec3::Y,
                bounding_circle.radius,
                color,
            );
    }
}
