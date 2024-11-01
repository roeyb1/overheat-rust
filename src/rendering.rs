use avian3d::prelude::{Position, Rotation};
use bevy::{core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping}, diagnostic::LogDiagnosticsPlugin, pbr::ScreenSpaceAmbientOcclusionBundle, prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt};
use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};
use bevy_sprite3d::{Sprite3d, Sprite3dParams, Sprite3dPlugin};
use lightyear::{client::prediction::diagnostics::PredictionDiagnosticsPlugin, prelude::client::{Confirmed, Predicted, VisualInterpolateStatus, VisualInterpolationPlugin}, transport::io::IoDiagnosticsPlugin};

use crate::{animation::{Animation, FaceCamera, OverheatAnimationPlugin}, assets::PlayerAssets, player::PlayerId, shared::GameState};

pub struct OverheatRenderPlugin;

impl Plugin for OverheatRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Sprite3dPlugin);

        app.add_plugins(OverheatAnimationPlugin);

        app.init_state::<GameState>();
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Game)
                .load_collection::<PlayerAssets>()
        );

        app.add_plugins(LogDiagnosticsPlugin {
            filter: Some(vec![
                IoDiagnosticsPlugin::BYTES_IN,
                IoDiagnosticsPlugin::BYTES_OUT,
            ]),
            ..default()
        });
        app.add_plugins(ScreenDiagnosticsPlugin::default());
        app.insert_resource(Msaa::Off);

        app.add_plugins(VisualInterpolationPlugin::<Position>::default());
        app.add_plugins(VisualInterpolationPlugin::<Rotation>::default());
        app.observe(add_visual_interpolation_components::<Position>);
        app.observe(add_visual_interpolation_components::<Rotation>);

        app.add_systems(Startup, (
            init,
            init_level_visuals,
        ));
        app.add_systems(Startup, setup_diagnostics);


        app.add_systems(Update, (
            init_player_visuals
            .run_if(in_state(GameState::Game)),
        ));

    }
}

fn init(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            //projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
            //    fov: PI / 6.,
            //    ..default()
            //}),
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(15.0),
                ..default()
            }.into(),
            transform: Transform::from_xyz(22., 18., 22.).looking_at(Vec3::splat(0.), Vec3::Y),
            ..default()
        },
        BloomSettings {
            intensity: 0.3,
            ..default()
        },
    ))
    .insert(ScreenSpaceAmbientOcclusionBundle::default());

    commands.spawn(Tonemapping::AcesFitted);
}

fn init_level_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // #todo: this is only spawning the rendering state of the world, we need a way to split render and physics and spawn this in a common way for server/client

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1_000_000.,
            color: Color::srgb(1., 231. / 255., 221. / 255.),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-2., 5., -2.),
        ..default()
    });

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

fn init_player_visuals(
    mut commands: Commands,
    mut sprite_params: Sprite3dParams,
    player_assets: Res<PlayerAssets>,
    query: Query<Entity, (With<PlayerId>, With<Predicted>, Without<PlayerVisualsMarker>)>,
) {
    for player in &query {
        let atlas = TextureAtlas {
            layout: player_assets.player_atlas.clone(),
            index: 0
        };

        let sprite = commands.spawn((
            Sprite3d {
                image: player_assets.player_tileset.clone(),
                pixels_per_metre: 16.,
                double_sided: false,
                pivot: Some(Vec2::new(0.5, 1. / 3.)),
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            }.bundle_with_atlas(&mut sprite_params, atlas),
            FaceCamera {},
            Animation {
                frames: vec![1, 2, 1, 0],
                current: 0,
                timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            },
        )).id();

        commands.entity(player).add_child(sprite);

        commands.entity(player).insert(PlayerVisualsMarker);
    }
}

fn setup_diagnostics(mut on_screen: ResMut<ScreenDiagnostics>) {
    on_screen
        .add(
            "Rollbacks".to_string(),
            PredictionDiagnosticsPlugin::ROLLBACKS,
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
    on_screen
        .add(
            "Rollback ticks".to_string(),
            PredictionDiagnosticsPlugin::ROLLBACK_TICKS
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
    on_screen
        .add(
            "RB Depth".to_string(),
            PredictionDiagnosticsPlugin::ROLLBACK_DEPTH,
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.1}"));
    on_screen
        .add("KB_in".to_string(), IoDiagnosticsPlugin::BYTES_IN)
        .aggregate(Aggregate::Average)
        .format(|v| format!("{v:0>3.0}"));
    on_screen
        .add("KB_out".to_string(), IoDiagnosticsPlugin::BYTES_OUT)
        .aggregate(Aggregate::Average)
        .format(|v| format!("{v:0>3.0}"));
}


fn add_visual_interpolation_components<T: Component>(
    trigger: Trigger<OnAdd, T>,
    mut commands: Commands,
    query: Query<Entity, (With<T>, Without<Confirmed>)>,
) {
    if !query.contains(trigger.entity()) {
        return;
    }

    commands
        .entity(trigger.entity())
        .insert(VisualInterpolateStatus::<T> {
            trigger_change_detection: true,
            ..default()
        });
}

#[derive(Component)]
struct PlayerVisualsMarker;