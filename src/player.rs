use bevy::prelude::*;
use leafwing_input_manager::{prelude::{ActionState, InputMap}, Actionlike, InputManagerBundle};
use lightyear::prelude::{client, ClientId, PrePredicted, ReplicateHierarchy, ReplicationGroup};
use serde::{Deserialize, Serialize};

use crate::physics::{CharacterQueryItem, PhysicsBundle};


pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

#[derive(Actionlike, Serialize, Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum PlayerActions {
    #[actionlike(DualAxis)]
    Move,
    Dodge,
    PrimaryAttack,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub ClientId);

#[derive(Component, Serialize, Deserialize, PartialEq, Reflect, Clone)]
pub struct MoveSpeed(pub f32);

#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    replicate: client::Replicate,
    inputs: InputManagerBundle<PlayerActions>,
    pre_predicted: PrePredicted,
    name: Name,

    spatial: SpatialBundle,
    physics: PhysicsBundle,

    // #todo: move
    move_speed: MoveSpeed,
}

impl PlayerBundle {
    pub fn new(id: ClientId, _position: Vec2, input_map: InputMap<PlayerActions>) -> Self {
        Self {
            id: PlayerId(id),
            replicate: client::Replicate {
                group: REPLICATION_GROUP,
                hierarchy: ReplicateHierarchy {
                    recursive: false,
                },
                ..default()
            },
            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            spatial: SpatialBundle::default(),
            physics: PhysicsBundle::player(),
            pre_predicted: PrePredicted::default(),
            move_speed: MoveSpeed(10.),
            name: Name::from("Player"),
        }
    }
}

pub fn shared_player_movement(
    time: &Res<Time>,
    move_speed: &MoveSpeed,
    action: &ActionState<PlayerActions>,
    character: &mut CharacterQueryItem,
) {
    use std::f32::consts::PI;

    const MAX_ACCEL: f32 = 20.;
    
    let max_velocity_delta_per_tick = MAX_ACCEL * time.delta_seconds();

    let input_dir = action.axis_pair(&PlayerActions::Move).clamp_length_max(1.);
    let move_dir = Vec2::from_angle(-PI / 4.).rotate(input_dir);
    let move_dir = Vec3::new(move_dir.x, 0., move_dir.y);

    let current_velocity = Vec3::new(character.linear_velocity.x, 0., character.linear_velocity.z);
    let desired_velocity = move_dir * move_speed.0;

    let new_velocity = current_velocity.move_towards(desired_velocity, max_velocity_delta_per_tick);

    let required_accel = (new_velocity - current_velocity) / time.delta_seconds();

    character.external_force.apply_force(required_accel * character.mass.0);
}