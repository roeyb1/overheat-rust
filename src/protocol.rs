use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use lightyear::{prelude::{client::ComponentSyncMode, AppComponentExt, ChannelDirection}, utils::avian3d::{position, rotation}};
use lightyear::shared::input::leafwing::LeafwingInputPlugin;

use crate::{abilities::Dodge, ability_framework::{ability_map::AbilityMap, cooldown::Cooldown, pool::AbilityCost, pools::{life::LifePool, mana::ManaPool}, Ability, AbilityCharge, PredictedAbility}, player::{CursorPosition, MoveSpeed, PlayerActions, PlayerId}};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LeafwingInputPlugin::<PlayerActions>::default());

        app.register_component::<Name>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<PlayerId>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<Position>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(position::lerp)
            .add_correction_fn(position::lerp);
        app.register_component::<Rotation>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(rotation::lerp)
            .add_correction_fn(rotation::lerp);

        app.register_component::<MoveSpeed>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Simple);

        app.register_component::<LinearVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);
        app.register_component::<AngularVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<CursorPosition>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<LifePool>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);
        app.register_component::<AbilityCost<LifePool>>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Simple);

        app.register_component::<ManaPool>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);
        app.register_component::<AbilityCost<ManaPool>>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Simple);

        //  sync ability states
        app.register_component::<Ability>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PredictedAbility>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<AbilityCharge>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<Cooldown>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);

        app.register_component::<AbilityMap<PlayerActions>>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple)
            .add_map_entities();

        // Ability tags
        app.register_component::<Dodge>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
    }
}