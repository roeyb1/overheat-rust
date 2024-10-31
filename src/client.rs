use bevy::prelude::*;
use bevy_asset_loader::loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt};
use bevy_sprite3d::{Sprite3d, Sprite3dParams, Sprite3dPlugin};
use leafwing_input_manager::prelude::{ActionState, InputMap, KeyboardVirtualDPad, WithDualAxisProcessingPipelineExt};
use lightyear::{prelude::{client::{ClientCommands, Interpolated, Predicted, PredictionSet}, MainSet}, shared::replication::components::Controlled};
use lightyear::client::events::*;

use crate::{assets::PlayerAssets, physics::{CharacterQuery, PhysicsBundle}, player::{shared_player_movement, PlayerActions, PlayerBundle}, shared::{FixedSet, GameState, MoveSpeed}};

pub struct OverheatClientPlugin;

impl Plugin for OverheatClientPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<GameState>()
        .add_plugins(Sprite3dPlugin)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Game)
                .load_collection::<PlayerAssets>()
        )
        .add_systems(Startup, init)
        .add_systems(
            PreUpdate,
            handle_connection
                .after(MainSet::Receive)
                .before(PredictionSet::SpawnPrediction)
                .run_if(in_state(GameState::Game))
        )
        .add_systems(FixedUpdate, 
            predicted_player_movement
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.connect_client();

    // #todo: this is only spawning the rendering state of the world, we need a way to split render and physics and spawn this in a common way for server/client
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(30., 30.)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(0.6)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(-1., 0.5, -1.),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(0.6)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(-1., 0.5, 7.),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(10., 2., 2.)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(-1., 0.5, 7.),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(2., 2., 10.)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(7., 0.5, -1.),
        ..default()
    });
}

fn handle_connection(
    mut commands: Commands,
    mut connection_event: EventReader<ConnectEvent>,
    mut sprite_params: Sprite3dParams,
    player_assets: Res<PlayerAssets>,
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

        let atlas = TextureAtlas {
            layout: player_assets.player_atlas.clone(),
            index: 0
        };

        let sprite = commands.spawn(
            (
                Sprite3d {
                    image: player_assets.player_tileset.clone(),
                    pixels_per_metre: 16.,
                    double_sided: false,
                    pivot: Some(Vec2::new(0.5, 1. / 3.)),
                    transform: Transform::from_xyz(0., 0., 0.),
                    ..default()
                }.bundle_with_atlas(&mut sprite_params, atlas),
                //Animation {
                //    frames: vec![1, 2, 1, 0],
                //    current: 0,
                //    timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                //},
                //FaceCamera {}
        )).id();

        info!("Handled new connection");
        commands.entity(player).add_child(sprite);

        info!("Attaching camera to locally controlled player");
        let camera_entity = cam_query.single();
        commands.entity(player).add_child(camera_entity);
    }
}

fn add_player_physics(
    mut commands: Commands,
    mut query: Query<(Entity, Has<Controlled>), Or<(Added<Interpolated>, Added<Predicted>)>>,
) {
    for (entity, locally_controlled) in query.iter_mut() {
        if locally_controlled {
            continue;
        }

        //only need to do this for remote players:
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