use bevy::prelude::*;
use leafwing_input_manager::prelude::{ActionState, InputMap, KeyboardVirtualDPad, WithDualAxisProcessingPipelineExt};
use lightyear::{prelude::{client::{ClientCommands, Interpolated, Predicted, PredictionSet}, MainSet}, shared::replication::components::Controlled};
use lightyear::client::events::*;

use crate::{physics::{CharacterQuery, PhysicsBundle}, player::{shared_player_movement, MoveSpeed, PlayerActions, PlayerBundle}, shared::FixedSet};

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
            predicted_player_movement
                .in_set(FixedSet::Main)
        )
        .add_systems(
            Update, (
                finalize_remote_player_spawn,
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
    mut connection_event: EventReader<ConnectEvent>,
    cam_query: Query<Entity, With<Camera3d>>,
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

        let player = commands.spawn((
            PlayerBundle::new(
                client_id,
                Vec2::new(0., 0.),
                InputMap::new([
                    (PlayerActions::Dodge, KeyCode::KeyW),
                ])
                .with_dual_axis(
                    PlayerActions::Move, KeyboardVirtualDPad::WASD
                        .inverted_y()
                ),
            ),
        )).id();

        let camera_entity = cam_query.single();
        commands.entity(player).add_child(camera_entity);
    }
}

/// Blueprint pattern: when the player is replicated from the server, it will only contain
/// the components which are always replicated. We need to add a few components that we don't
/// need to be replicated all the time, for example the physics data which is constant and
/// shouldn't be constantly replicated.
fn finalize_remote_player_spawn(
    mut commands: Commands,
    mut query: Query<(Entity, Has<Controlled>), Or<(Added<Interpolated>, Added<Predicted>)>>,
) {
    for (entity, locally_controlled) in query.iter_mut() {
        //only need to do this for remote players:
        if locally_controlled {
            continue;
        }

        commands.entity(entity).insert(SpatialBundle::default());

        commands.entity(entity).insert(PhysicsBundle::player());
    }
}

fn predicted_player_movement(
    time: Res<Time>,
    mut query: Query<(CharacterQuery, &MoveSpeed, &ActionState<PlayerActions>), With<Predicted>>,
) {
    for (mut character, move_speed, action_state) in &mut query {
        shared_player_movement(&time, move_speed, action_state, &mut character);
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