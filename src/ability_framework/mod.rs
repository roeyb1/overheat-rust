use std::time::Duration;

use bevy::{ecs::query::QueryData, prelude::*};
use cooldown::Cooldown;
use lightyear::prelude::client::Predicted;
use pool::{predict_tick_pools_regen, tick_pools_regen, AbilityCost, Pool};
use pools::{life::{Life, LifePool}, mana::{Mana, ManaPool}};
use serde::{Deserialize, Serialize};

pub mod cooldown;
pub mod pool;
pub mod pools;
pub mod ability_map;

pub struct AbilityFrameworkServerPlugin;

impl Plugin for AbilityFrameworkServerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_systems(FixedUpdate, (
            tick_pools_regen::<LifePool>,
            tick_pools_regen::<ManaPool>,
            tick_ability_cds,
            tick_ability_charge,
        ));
    }
}

pub struct AbilityFrameworkClientPlugin;

impl Plugin for AbilityFrameworkClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_systems(FixedUpdate, (
            predict_tick_pools_regen::<LifePool>,
            predict_tick_pools_regen::<ManaPool>,
            predict_tick_ability_cds,
            predict_tick_ability_charge,
        ));
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct Ability;

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct PredictedAbility;

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Default, Reflect)]
pub struct AbilityCharge(pub Duration);

impl AbilityCharge {
    pub fn start(&mut self) {
        self.0 = Duration::ZERO;
    }

    pub fn duration(&self) -> Duration {
        self.0
    }
}

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

#[allow(unused)]
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
pub struct TriggerAbility {
    /// Source entity that triggered the ability
    pub source: Entity,
    /// Entity which describes the ability being triggered
    pub ability: Entity,
}

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

fn tick_ability_charge(
    time: Res<Time>,
    mut query: Query<&mut AbilityCharge>
) {
    for mut charge in query.iter_mut() {
        charge.0 += time.delta();
    }
}

fn predict_tick_ability_cds(
    time: Res<Time>,
    mut query: Query<&mut Cooldown, (With<Ability>, With<Predicted>)>
) {
    for mut cd in query.iter_mut() {
        cd.tick(time.delta());
    }
}

fn predict_tick_ability_charge(
    time: Res<Time>,
    mut query: Query<&mut AbilityCharge, (With<PredictedAbility>, With<Predicted>)>
) {
    for mut charge in query.iter_mut() {
        charge.0 += time.delta();
    }
}