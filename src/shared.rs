use avian3d::prelude::*;
use bevy::{prelude::*, render::RenderPlugin};

use crate::{abilities::AbilitiesPlugin, ability_framework::{ability_map::AbilityMap, pool::AbilityCost, pools::{life::LifePool, mana::ManaPool}, TriggerAbility}, player::{CursorPosition, PlayerActions, PlayerId}, protocol::ProtocolPlugin, rendering::OverheatRenderPlugin, FIXED_TIMESTEP_HZ};

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
        app.add_plugins(AbilitiesPlugin);

        app.add_event::<TriggerAbility>();

        if app.is_plugin_added::<RenderPlugin>() {
            app.add_plugins(OverheatRenderPlugin);
        }

        // Position and Rotation are the primary source of truth so no need to
        // sync changes from Transform to Position.
        app.insert_resource(avian3d::sync::SyncConfig {
            transform_to_position: false,
            position_to_transform: true,
        });

        app.add_systems(Startup, init);

        // SyncPlugin should be disabled then manually re-enabled so it synchronizes every frame in post update to capture interpolated values
        app.add_plugins(
            PhysicsPlugins::new(FixedUpdate)
                .build()
                .disable::<SyncPlugin>()
                .disable::<ColliderHierarchyPlugin>(),
        )
        .add_plugins(SyncPlugin::new(PostUpdate))

        .insert_resource(Time::new_with(Physics::fixed_once_hz(FIXED_TIMESTEP_HZ)))
        .insert_resource(Gravity(Vec3::ZERO));

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
        app.register_type::<CursorPosition>();
        app.register_type::<AbilityMap<PlayerActions>>();
        app.register_type::<LifePool>();
        app.register_type::<ManaPool>();
        app.register_type::<AbilityCost<LifePool>>();
        app.register_type::<AbilityCost<ManaPool>>();
    }
}

fn init() {

}
