use bevy::prelude::*;

use std::collections::BTreeSet;
use std::cmp::Ordering;

/// Hatred.
///
/// No, not the messy, visceral kind. When a [`Targeting`] system finds more
/// valid targets than it can store, this is how it determines which ones are
/// more important to target. Higher hatred means it is prioritized, if no
/// taunt shenanigans are currently happening.
///
/// **For enemies:**  
/// Hatred is how close the enemy is to the end of their pathing, multiplied by
/// -1,000. If an enemy is about 6 tiles from the end of their pathing, their
/// resulting `Hatred` will be -6000.
///
/// **For allies:**  
/// Hatred is which number operator this operator was deployed. Later deployed
/// operators means they have a higher `Hatred` value.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Hatred(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SortedEntity {
    entity: Entity,
    hatred: Hatred,
}

impl PartialOrd for SortedEntity {
    fn partial_cmp(&self, other: &SortedEntity) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortedEntity {
    fn cmp(&self, other: &SortedEntity) -> Ordering {
        // first sort by hatred
        let order = self.hatred.cmp(&other.hatred).reverse();

        match order {
            // order by entity
            Ordering::Equal => self.entity.cmp(&other.entity).reverse(),
            order => order,
        }
    }
}

/// The targeting tree of entities.
#[derive(Clone, Debug, Default, Resource)]
pub struct TargetingTree {
    tree: BTreeSet<SortedEntity>,
}

impl TargetingTree {
    /// Iters over each target.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        self.tree.iter().map(|se| se.entity)
    }
}

pub fn sort_targets(
    query: Query<(Entity, &Hatred)>,
    mut hatred_tree: ResMut<TargetingTree>,
) {
    hatred_tree.tree.clear();

    for (entity, hatred) in query.iter() {
        hatred_tree.tree.insert(SortedEntity {
            entity,
            hatred: *hatred,
        });
    }
}
