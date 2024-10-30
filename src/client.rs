use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use leafwing_input_manager::prelude::{ActionState, InputMap, KeyboardVirtualDPad, WithDualAxisProcessingPipelineExt};
use lightyear::prelude::{client::{ClientCommands, ClientConnection, Interpolated, NetClient, Predicted, PredictionSet}, MainSet};
use lightyear::client::events::*;

use crate::{physics::PhysicsBundle, player::{shared_player_movement, PlayerActions, PlayerBundle, PlayerId}, shared::{FixedSet, MoveSpeed}};

pub struct OverheatClientPlugin;

impl Plugin for OverheatClientPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, init)
        .add_systems(
            PreUpdate,
            handle_connection
                .after(MainSet::Receive)
                .before(PredictionSet::SpawnPrediction)
        )
        .add_systems(FixedUpdate, 
            player_movement
                .in_set(FixedSet::Main)
        )
        .add_systems(
            Update, (
                add_player_physics,
                handle_predicted_spawn,
                handle_interpolated_spawn,
            )
        );
    }
}

fn init(
    mut commands: Commands,
) {
    commands.connect_client();
}

fn handle_connection(
    mut commands: Commands,
    mut connection_event: EventReader<ConnectEvent>
) {
    for event in connection_event.read() {
        let client_id = event.client_id();
        commands.spawn(TextBundle::from_section(
            format!("Client {}", client_id),
            TextStyle {
                font_size: 30.,
                color: Color::WHITE,
                ..default()
            },
        ));

        commands.spawn(PlayerBundle::new(
            client_id,
            Vec2::new(0., 0.),
            InputMap::new([
                (PlayerActions::Dodge, KeyCode::KeyW),
            ])
            .with_dual_axis(
                PlayerActions::Move, KeyboardVirtualDPad::WASD
                    .with_circle_deadzone(0.1)
                    .inverted_y()
            )
        ));
    }
}

fn add_player_physics(
    mut commands: Commands,
    connection: Res<ClientConnection>,
    mut query: Query<(Entity, &PlayerId), Or<(Added<Interpolated>, Added<Predicted>)>>,
) {
    let client_id = connection.id();
    for (entity, player_id) in query.iter_mut() {
        if player_id.0 == client_id {
            continue;
        }

        //only need to do this for remote players:
        commands.entity(entity).insert(PhysicsBundle::player());
    }
}

fn player_movement(
    mut query: Query<(&mut LinearVelocity, &MoveSpeed, &ActionState<PlayerActions>), With<Predicted>>,
) {
    for (velocity, speed, action_state) in query.iter_mut() {
        shared_player_movement(velocity, speed, action_state);
    }
}

fn handle_predicted_spawn(
    query: Query<Entity, Added<Predicted>>
) {
    for entity in &query {
        info!("New predicted entity spawned: {}", entity)
    }
}

fn handle_interpolated_spawn(
    query: Query<Entity, Added<Interpolated>>
) {
    for entity in &query {
        info!("New interpolated entity spawned: {}", entity)
    }

}