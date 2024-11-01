use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::prelude::*;
use lightyear::{prelude::{client::ComponentSyncMode, AppComponentExt, ChannelDirection}, utils::avian3d::{position, rotation}};
use lightyear::shared::input::leafwing::LeafwingInputPlugin;

use crate::{ability::pools::life::LifePool, player::{MoveSpeed, PlayerActions, PlayerId}};

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

        app.register_component::<LifePool>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);
    }
}