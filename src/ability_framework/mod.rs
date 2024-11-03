use std::time::Duration;

use bevy::{ecs::query::QueryData, prelude::*};
use cooldown::Cooldown;
use pool::{tick_pools_regen, AbilityCost, Pool};
use pools::{life::{Life, LifePool}, mana::{Mana, ManaPool}};
use serde::{Deserialize, Serialize};

pub mod cooldown;
pub mod pool;
pub mod pools;
pub mod ability_map;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_systems(FixedUpdate, (
            tick_pools_regen::<LifePool>,
            tick_pools_regen::<ManaPool>,
            tick_ability_cds,
        ));
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct Ability;

#[derive(Bundle)]
pub struct AbilityBundle {
    ability: Ability,
    mp_cost: AbilityCost<ManaPool>,
    lp_cost: AbilityCost<LifePool>,
    cooldown: Cooldown,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct AbilityState {
    pub ability: &'static mut Ability,
    pub cooldown: &'static mut Cooldown,
    pub mp_cost: Option<&'static mut AbilityCost<ManaPool>>,
    pub lp_cost: Option<&'static mut AbilityCost<LifePool>>,
}

impl AbilityBundle {
    pub fn new(mp_cost: f32, life_cost: f32, cooldown: Duration) -> Self {
        Self {
            ability: Ability,
            mp_cost: AbilityCost::<ManaPool>(Mana(mp_cost)),
            lp_cost: AbilityCost::<LifePool>(Life(life_cost)),
            cooldown: Cooldown::from_secs(cooldown.as_secs_f32()),
        }
    }
}
impl AbilityStateReadOnlyItem<'_> {
    pub fn ready(&self, mana: &ManaPool, life: &LifePool) -> Result<(), CannotUseAbility> {
        self.cooldown.ready()?;

        let maybe_mp_cost = self.mp_cost.as_deref();
        if let Some(mana_cost) = maybe_mp_cost {
            mana.available(mana_cost.0)?;
        }

        let maybe_lp_cost = self.lp_cost.as_deref();
        if let Some(life_cost) = maybe_lp_cost {
            life.available(life_cost.0)?;
        }

        Ok(())
    }
}

impl AbilityStateItem<'_> {
    pub fn ready(&self, mana: &ManaPool, life: &LifePool) -> Result<(), CannotUseAbility> {
        self.cooldown.ready()?;

        let maybe_mp_cost = self.mp_cost.as_deref();
        if let Some(mana_cost) = maybe_mp_cost {
            mana.available(mana_cost.0)?;
        }

        let maybe_lp_cost = self.lp_cost.as_deref();
        if let Some(life_cost) = maybe_lp_cost {
            life.available(life_cost.0)?;
        }

        Ok(())
    }

    pub fn trigger(&mut self, mana: &mut ManaPool, life: &mut LifePool) -> Result<(), CannotUseAbility> {
        self.ready(mana, life)?;

        self.cooldown.trigger()?;

        let maybe_mp_cost = self.mp_cost.as_deref();
        if let Some(mana_cost) = maybe_mp_cost {
            mana.expend(mana_cost.0)?;
        }

        let maybe_lp_cost = self.lp_cost.as_deref();
        if let Some(life_cost) = maybe_lp_cost {
            life.expend(life_cost.0)?;
        }

        Ok(())
    }
}

#[derive(Event)]
pub struct TriggerAbility(pub Entity);

#[derive(Debug)]
pub enum CannotUseAbility {
    OnCooldown,
    ResourceMissing,
    AbilityNotBound,
}


fn tick_ability_cds(
    time: Res<Time>,
    mut query: Query<&mut Cooldown, With<Ability>>
) {
    for mut cd in query.iter_mut() {
        cd.tick(time.delta());
    }
}