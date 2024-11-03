use bevy::{ecs::entity::MapEntities, prelude::*, utils::HashMap};
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

use super::CannotUseAbility;

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct AbilityMap<A: Actionlike> {
    bindings: HashMap<A, Entity>,
}

impl<A: Actionlike> AbilityMap<A> {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn add_binding(&mut self, action: A, entity: Entity) {
        self.bindings.insert(action, entity);
    }

    pub fn mapped(&self, action: A) -> Result<Entity, CannotUseAbility> {
        match self.bindings.get(&action) {
            Some(ability) => {
                Ok(ability.clone())
            },
            None => Err(CannotUseAbility::AbilityNotBound)
        }
    }
}

impl<A: Actionlike> MapEntities for AbilityMap<A> {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in self.bindings.values_mut() {
            let mapped = entity_mapper.map_entity(*entity);
            *entity = mapped;
        }
    }
}