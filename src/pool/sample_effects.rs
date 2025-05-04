use bevy::{
    ecs::query::{QueryData, QueryFilter, QueryManyIter, QueryManyUniqueIter, ROQueryItem},
    prelude::*,
};

#[derive(Debug, Component)]
#[relationship(relationship_target = SampleEffects)]
pub struct EffectOf(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct SampleEffects(super::entity_set::EffectsSet);

impl core::ops::Deref for SampleEffects {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
pub use bevy::ecs::spawn::Spawn;

use super::entity_set::EffectsSetIter;

#[macro_export]
macro_rules! sample_effects {
    [$($effect:expr),*$(,)?] => {
        <$crate::pool::sample_effects::SampleEffects>::spawn(($($crate::pool::sample_effects::Spawn($effect)),*))
    };
}

#[derive(Debug)]
pub enum EffectsQueryError {
    MatchedMultiple,
    MatchedNone,
}

impl core::fmt::Display for EffectsQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MatchedMultiple => write!(f, "audio effects query matched multiple entities"),
            Self::MatchedNone => write!(f, "audio effects query matched no entities"),
        }
    }
}

impl core::error::Error for EffectsQueryError {}

pub trait EffectsQuery<'s, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    fn get_effect(&self, effects: &SampleEffects) -> Result<ROQueryItem<'_, D>, EffectsQueryError>;

    fn get_effect_mut(&mut self, effects: &SampleEffects)
    -> Result<D::Item<'_>, EffectsQueryError>;

    fn iter_effects<'a>(
        &self,
        effects: &'a SampleEffects,
    ) -> QueryManyUniqueIter<'_, 's, D::ReadOnly, F, EffectsSetIter<'a>>;

    fn iter_mut(
        &mut self,
        effects: &SampleEffects,
    ) -> QueryManyIter<'_, 's, D, F, impl Iterator<Item = Entity>>;
}

impl<'s, D, F> EffectsQuery<'s, D, F> for Query<'_, 's, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    fn get_effect(&self, effects: &SampleEffects) -> Result<ROQueryItem<'_, D>, EffectsQueryError> {
        if self.iter_many_unique(effects.iter()).count() > 1 {
            return Err(EffectsQueryError::MatchedMultiple);
        }

        self.iter_many_unique(effects.iter())
            .next()
            .ok_or(EffectsQueryError::MatchedNone)
    }

    fn get_effect_mut(
        &mut self,
        effects: &SampleEffects,
    ) -> Result<D::Item<'_>, EffectsQueryError> {
        if self.iter_many_unique(effects.iter()).count() > 1 {
            return Err(EffectsQueryError::MatchedMultiple);
        }

        self.iter_many_unique_mut(effects.iter())
            .next()
            .ok_or(EffectsQueryError::MatchedNone)
    }

    fn iter_effects<'a>(
        &self,
        effects: &'a SampleEffects,
    ) -> QueryManyUniqueIter<'_, 's, D::ReadOnly, F, EffectsSetIter<'a>> {
        self.iter_many_unique(effects.iter())
    }

    fn iter_mut(
        &mut self,
        effects: &SampleEffects,
    ) -> QueryManyIter<'_, 's, D, F, impl Iterator<Item = Entity>> {
        self.iter_many_mut(effects.iter())
    }
}
