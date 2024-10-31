use bevy::prelude::*;
use pool::tick_pools_regen;
use pools::life::LifePool;

pub mod cooldown;
pub mod pool;
pub mod pools;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_systems(Update,
            tick_pools_regen::<LifePool>
        );
    }
}

#[derive(Component)]
pub struct Ability;

#[derive(Debug)]
pub enum CannotUseAbility {
    OnCooldown,
    ResourceMissing,
    AbilityNotBound,
}
