use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::{server::{AuthorityPeer, ControlledBy, Replicate, ServerCommands, ServerReplicationSet, SyncTarget}, InputChannel, InputMessage, MainSet, NetworkTarget, OverrideTargetComponent, PrePredicted, Replicated, ReplicationTarget};
use lightyear::server::{connection::ConnectionManager, events::MessageEvent};

use crate::{physics::{CharacterQuery, PhysicsBundle}, player::{shared_player_movement, CursorPosition, MoveSpeed, PlayerActions, PlayerId, REPLICATION_GROUP}, shared::FixedSet};

pub struct OverheatServerPlugin {
    pub predict_all: bool,
}

#[derive(Resource)]
pub struct Global {
    predict_all: bool,
}

impl Plugin for OverheatServerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(Global {
            predict_all: self.predict_all
        })
        .add_systems(Startup, (start_server, init))
        .add_systems(
            PreUpdate,
            replicate_inputs
                .after(MainSet::EmitEvents)
        )
        .add_systems(
            PreUpdate, (
                replicate_players,
                replicate_cursors,
            ).in_set(ServerReplicationSet::ClientReplication)
        )
        .add_systems(
            FixedUpdate,
            movement
                .in_set(FixedSet::Main)
        );
    }
}

fn start_server(
    mut commands: Commands
) {
    commands.start_server();
}

fn init(
    mut commands: Commands,
) {
    commands.spawn(TextBundle::from_section(
        "Server",
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        })
        .with_style(Style {
            align_self: AlignSelf::End,
            ..default()
        })
    );

}

fn replicate_inputs(
    mut connection: ResMut<ConnectionManager>,
    mut input_events: ResMut<Events<MessageEvent<InputMessage<PlayerActions>>>>,
) {
    for mut event in input_events.drain() {
        let client_id = *event.context();

        connection
            .send_message_to_target::<InputChannel, _>(
                &mut event.message,
                NetworkTarget::AllExceptSingle(client_id)
            )
            .unwrap()
    }
}

fn replicate_players(
    mut commands: Commands,
    global: Res<Global>,
    query: Query<(Entity, &Replicated), (Added<Replicated>, With<PlayerId>)>,
) {
    for (entity, replicated) in query.iter() {
        let client_id = replicated.client_id();
        info!("received player spawn event from client {client_id:?}");

        if let Some(mut e) = commands.get_entity(entity) {
            let mut sync_target = SyncTarget::default();

            if global.predict_all {
                sync_target.prediction = NetworkTarget::All;
            } else {
                sync_target.interpolation = NetworkTarget::AllExceptSingle(client_id);
            }

            let replicate = Replicate {
                sync: sync_target,
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(client_id),
                    ..default()
                },
                group: REPLICATION_GROUP,
                ..default()
            };
            e.insert((
                replicate,
                OverrideTargetComponent::<PrePredicted>::new(NetworkTarget::Single(client_id)),
                PhysicsBundle::player(),
            ));
        }
    }
}

fn replicate_cursors(
    mut commands: Commands,
    query: Query<(Entity, &Replicated), (Added<Replicated>, With<CursorPosition>)>,
) {
    for (entity, replicated) in &query {
        let client_id = replicated.client_id();

        if let Some(mut e) = commands.get_entity(entity) {
            e.insert(Replicate {
                target: ReplicationTarget {
                    // don't need to replicate this back to the client that owns it
                    target: NetworkTarget::AllExceptSingle(client_id),
                },
                authority: AuthorityPeer::Client(client_id),
                sync: SyncTarget {
                    // other clients should interpolate the cursor pos
                    interpolation: NetworkTarget::AllExceptSingle(client_id),
                    ..default()
                },
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(client_id),
                    ..default()
                },
                ..default()
            });
        }
    }
}

fn movement(
    time: Res<Time>,
    mut query: Query<(CharacterQuery, &MoveSpeed, &ActionState<PlayerActions>)>,
) {
    for (mut character, move_speed, action_state) in &mut query {
        shared_player_movement(&time, move_speed, action_state, &mut character);
    }
}