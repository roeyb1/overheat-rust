use avian2d::prelude::*;
use bevy::{core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping}, diagnostic::LogDiagnosticsPlugin, pbr::ScreenSpaceAmbientOcclusionBundle, prelude::*, render::{camera::ScalingMode, RenderPlugin}};
use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};
use bevy_sprite3d::Sprite3dPlugin;
use lightyear::{client::prediction::diagnostics::PredictionDiagnosticsPlugin, prelude::client::{Confirmed, InterpolationSet, PredictionSet}, transport::io::IoDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

use crate::{player::PlayerId, protocol::ProtocolPlugin, FIXED_TIMESTEP_HZ};

pub struct OverheatSharedPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FixedSet {
    Main,
    Physics,
}

#[derive(States, Hash, Debug, PartialEq, Eq, Clone, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    Game,
}

impl Plugin for OverheatSharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        
        if app.is_plugin_added::<RenderPlugin>() {
            app.add_systems(Startup, init_camera);

            app.add_systems(
                PostUpdate,
                debug_draw
                    .after(InterpolationSet::Interpolate)
                    .after(PredictionSet::VisualCorrection)
            );

            app.add_plugins(LogDiagnosticsPlugin {
                filter: Some(vec![
                    IoDiagnosticsPlugin::BYTES_IN,
                    IoDiagnosticsPlugin::BYTES_OUT,
                ]),
                ..default()
            });
            app.add_systems(Startup, setup_diagnostics);
            app.add_plugins(ScreenDiagnosticsPlugin::default());
            app.insert_resource(Msaa::Off);
        }

        app.add_systems(Startup, init);
        app.add_plugins(
            PhysicsPlugins::new(FixedUpdate)
                .build()
                .disable::<ColliderHierarchyPlugin>(),
        )
        .insert_resource(Time::new_with(Physics::fixed_once_hz(FIXED_TIMESTEP_HZ)))
        .insert_resource(Gravity(Vec2::ZERO));

        app.configure_sets(
            FixedUpdate, (
                //ensure that physics simulation happens after the FixedSet::main which is where player input is handled
                (
                    PhysicsSet::Prepare,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Sync,
                ).in_set(FixedSet::Physics),
                (FixedSet::Main, FixedSet::Physics).chain()
            ),
        );

        app.register_type::<PlayerId>();
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Reflect, Clone)]
pub struct MoveSpeed(pub f32);

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

fn init_camera(mut commands: Commands) {
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
            transform: Transform::from_xyz(22., 22., 22.).looking_at(Vec3::splat(0.), Vec3::Y),
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

fn init() {

}

fn debug_draw(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation), (Without<Confirmed>, With<PlayerId>)>
) {
    for (position, rotation) in &players {
        gizmos.rect_2d(Vec2::new(position.x, position.y), rotation.as_radians(), Vec2::ONE * 40., Color::WHITE);
    }
}