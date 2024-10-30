use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::{prelude::{ActionState, InputMap}, Actionlike, InputManagerBundle};
use lightyear::prelude::{client, ClientId, PrePredicted, ReplicationGroup};
use serde::{Deserialize, Serialize};

use crate::{physics::PhysicsBundle, shared::MoveSpeed};


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

#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    replicate: client::Replicate,
    inputs: InputManagerBundle<PlayerActions>,
    pre_predicted: PrePredicted,
    name: Name,

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
                ..default()
            },
            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            physics: PhysicsBundle::player(),
            pre_predicted: PrePredicted::default(),
            move_speed: MoveSpeed(10.),
            name: Name::from("Player"),
        }
    }
}

pub fn shared_player_movement(
    mut velocity: Mut<LinearVelocity>,
    move_speed: &MoveSpeed,
    action: &ActionState<PlayerActions>,
) {
    use std::f32::consts::PI;

    let move_direction = action.clamped_axis_pair(&PlayerActions::Move);

    if move_direction != Vec2::ZERO {
        let iso_dir = Vec2::from_angle(-PI / 4.).rotate(move_direction);
        velocity.0 += move_speed.0 * iso_dir;
    }
}