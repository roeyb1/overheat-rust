use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;
use leafwing_input_manager::prelude::{ActionState, InputMap, KeyboardVirtualDPad, WithDualAxisProcessingPipelineExt};
use lightyear::{prelude::{client::{ClientCommands, Confirmed, Interpolated, Predicted, PredictionSet, Replicate}, MainSet}, shared::replication::components::Controlled};
use lightyear::client::events::*;

use crate::{ability_framework::{ability_map::AbilityMap, pools::{life::LifePool, mana::ManaPool}, AbilityCharge, AbilityFrameworkClientPlugin, AbilityState, PredictedAbility, TriggerAbility}, physics::{CharacterQuery, PhysicsBundle}, player::{shared_player_movement, CursorBundle, CursorPosition, MoveSpeed, PlayerActions, PlayerBundle, PlayerId}, shared::FixedSet};

pub struct OverheatClientPlugin;

impl Plugin for OverheatClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FilterQueryInspectorPlugin::<With<Predicted>>::default());
        app.add_plugins(FilterQueryInspectorPlugin::<With<Confirmed>>::default());

        app.add_plugins(AbilityFrameworkClientPlugin);
        app.add_systems(Startup, init);
        app.add_systems(
            PreUpdate,
            handle_connection
                .after(MainSet::Receive)
                .before(PredictionSet::SpawnPrediction)
        );
        app.add_systems(FixedUpdate, (
                predicted_player_movement,
                start_charging_predicted_abilities,
                trigger_predicted_abilities,
            )
            .in_set(FixedSet::Main)
        );
        app.add_systems(
            Update, (
                finalize_remote_player_spawn,
                handle_predicted_spawn,
                handle_interpolated_spawn,
                cursor_movement,
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
                    (PlayerActions::Dodge, KeyCode::Space),
                ])
                .with_multiple([
                    (PlayerActions::PrimaryAttack, MouseButton::Left),
                ])
                .with_dual_axis(
                    PlayerActions::Move, KeyboardVirtualDPad::WASD
                        .inverted_y()
                ),
            ),
        )).id();

        let camera_entity = cam_query.single();
        commands.entity(player).add_child(camera_entity);

        commands.spawn(CursorBundle {
            position: CursorPosition(Vec3::ZERO),
            replicate: Replicate::default(),
        });
    }
}

/// Blueprint pattern: when the player is replicated from the server, it will only contain
/// the components which are always replicated. We need to add a few components that we don't
/// need to be replicated all the time, for example the physics data which is constant and
/// shouldn't be constantly replicated.
fn finalize_remote_player_spawn(
    mut commands: Commands,
    mut query: Query<(Entity, Has<Controlled>), (With<PlayerId>, Or<(Added<Interpolated>, Added<Predicted>)>)>,
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

fn cursor_movement(
    window_query: Query<&Window>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cusor_query: Query<&mut CursorPosition, With<Controlled>>,
) {
    if let Ok(window) = window_query.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok((cam_transform, cam)) = camera_query.get_single() {
                if let Some(ray) = cam.viewport_to_world(&cam_transform, cursor_pos) {
                    let t = ray.intersect_plane(Vec3::splat(0.), InfinitePlane3d::new(Vec3::new(0.0, 1., 0.)));

                    if let Some(t) = t {
                        for mut cursor_pos in cusor_query.iter_mut() {
                            cursor_pos.set_if_neq(CursorPosition(ray.get_point(t)));
                        }
                    }
                }
            }
        }
    }
}

fn start_charging_predicted_abilities(
    action_query: Query<(&ActionState<PlayerActions>, &AbilityMap<PlayerActions>, &LifePool, &ManaPool), With<Predicted>>,
    mut ability_query: Query<(AbilityState, &mut AbilityCharge), (With<PredictedAbility>, With<Predicted>)>,
) {
    for (actions, map, life, mana) in action_query.iter() {
        for pressed in actions.get_just_pressed() {
            if let Ok(ability_entity) = map.mapped(pressed) {
                if let Ok((ability_state, mut charge)) = ability_query.get_mut(ability_entity) {
                    if ability_state.ready(mana, life).is_ok() {
                        charge.start();
                    }
                }
            }
        }
    }
}

fn trigger_predicted_abilities(
    mut action_query: Query<(Entity, &ActionState<PlayerActions>, &AbilityMap<PlayerActions>, &mut LifePool, &mut ManaPool), With<Predicted>>,
    mut triggers: EventWriter<TriggerAbility>,
    mut ability_query: Query<AbilityState, (With<PredictedAbility>, With<Predicted>)>,
) {
    for (entity, actions, map, mut life, mut mana) in action_query.iter_mut() {
        for pressed in actions.get_just_released() {
            if let Ok(ability_entity) = map.mapped(pressed) {
                if let Ok(mut ability) = ability_query.get_mut(ability_entity) {

                    match ability.trigger(&mut mana, &mut life) {
                        Ok(()) => {
                            triggers.send(TriggerAbility {
                                source: entity,
                                ability: ability_entity,
                            });
                        },
                        Err(_) => {},
                    }
                }
            }
        }
    }
}