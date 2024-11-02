use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

use super::CannotUseAbility;

#[derive(Component, Serialize, Deserialize, PartialEq, Clone)]
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

