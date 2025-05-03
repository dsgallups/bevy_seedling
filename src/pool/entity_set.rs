use std::iter::Copied;

use bevy::{
    ecs::{
        entity::{EntitySetIterator, MapEntities},
        relationship::RelationshipSourceCollection,
    },
    prelude::*,
};

/// A thin wrapper around `std::vec::Vec<Entity>`.
///
/// This type guarantees that all elements are unique.
#[derive(Debug)]
pub struct EffectsSet(Vec<Entity>);

impl EffectsSet {
    fn has_duplicates(&self) -> bool {
        for i in 1..self.len() {
            if self[i..].contains(&self[i - 1]) {
                return true;
            }
        }
        false
    }
}

impl MapEntities for EffectsSet {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for entity in self.0.iter_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }

        // Now, verify all items are unique.
        if self.has_duplicates() {
            panic!("`EntityMapper` produced duplicate elements in `EffectsSet`");
        }
    }
}

impl core::ops::Deref for EffectsSet {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RelationshipSourceCollection for EffectsSet {
    type SourceIter<'a> = EffectsSetIter<'a>;

    fn new() -> Self {
        EffectsSet(Vec::new())
    }

    fn with_capacity(capacity: usize) -> Self {
        EffectsSet(Vec::with_capacity(capacity))
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    fn add(&mut self, entity: Entity) -> bool {
        if self.0.contains(&entity) {
            return false;
        }

        self.0.push(entity);
        true
    }

    fn remove(&mut self, entity: Entity) -> bool {
        if let Some(index) = self.iter().position(|e| e == entity) {
            self.0.remove(index);
            return true;
        }

        false
    }

    fn iter(&self) -> Self::SourceIter<'_> {
        EffectsSetIter {
            iter: Vec::iter(&self.0),
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
}

#[derive(Debug)]
pub struct EffectsSetIter<'a> {
    iter: Copied<core::slice::Iter<'a, Entity>>,
}

impl Iterator for EffectsSetIter<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// # Safety
///
/// Because [`EffectsSet`] cannot be mutated in any way
/// that will introduce duplicate elements, this must be safe.
unsafe impl EntitySetIterator for EffectsSetIter<'_> {}
