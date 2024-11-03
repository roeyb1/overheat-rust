use std::time::Duration;

use bevy::prelude::*;
use cooldown::Cooldown;
use pool::{tick_pools_regen, AbilityCost};
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
            tick_ability_cds,
        ));
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Reflect)]
pub struct Ability {
    pub mp_cost: AbilityCost<ManaPool>,
    pub lp_cost: AbilityCost<LifePool>,

    pub cooldown: Cooldown,
}

impl Ability {
    pub fn new(mp_cost: f32, life_cost: f32, cooldown: Duration) -> Self {
        Self {
            mp_cost: AbilityCost::<ManaPool>(Mana(mp_cost)),
            lp_cost: AbilityCost::<LifePool>(Life(life_cost)),
            cooldown: Cooldown::from_secs(cooldown.as_secs_f32()),
        }
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
    mut query: Query<&mut Ability>
) {
    for mut ability in query.iter_mut() {
        ability.cooldown.tick(time.delta());
    }
}