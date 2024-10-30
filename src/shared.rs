use avian2d::prelude::*;
use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*, render::RenderPlugin};
use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};
use lightyear::{client::prediction::diagnostics::PredictionDiagnosticsPlugin, prelude::client::{Confirmed, InterpolationSet, PredictionSet}, transport::io::IoDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

use crate::{player::PlayerId, protocol::ProtocolPlugin, FIXED_TIMESTEP_HZ};

pub struct OverheatSharedPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FixedSet {
    Main,
    Physics,
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

#[derive(Component, Serialize, Deserialize)]
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
    commands.spawn(Camera2dBundle::default());
}

fn init(mut _commands: Commands) {
    //#todo: load the world
}

fn debug_draw(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation), (Without<Confirmed>, With<PlayerId>)>
) {
    for (position, rotation) in &players {
        gizmos.rect_2d(Vec2::new(position.x, position.y), rotation.as_radians(), Vec2::ONE * 40., Color::WHITE);
    }
}