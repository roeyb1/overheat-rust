use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{ability_framework::{Ability, AbilityCharge, TriggerAbility}, physics::CharacterQuery, player::MoveSpeed, shared::FixedSet};

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
                handle_dodge,
            ).in_set(FixedSet::Main)
        );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Dodge;

fn handle_dodge(
    mut events: EventReader<TriggerAbility>,
    dash_query: Query<&AbilityCharge, (With<Dodge>, With<Ability>)>,
    mut character_query: Query<(CharacterQuery, &MoveSpeed)>,
) {
    for trigger in events.read() {
        if let Ok(charge) = dash_query.get(trigger.ability) {
            if let Ok((mut character, speed)) = character_query.get_mut(trigger.source) {
                let move_dir = character.linear_velocity.normalize_or_zero();

                let base_distance = speed.0 * 3.;
                // multiply the base distance for every additional second the charge is held.
                let charge_factor = charge.duration().as_secs_f32() + 1.;
                character.external_impulse.apply_impulse(move_dir.normalize_or_zero() * base_distance * charge_factor);
            }
        }
    }
}