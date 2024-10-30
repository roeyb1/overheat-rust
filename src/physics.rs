use avian2d::prelude::{Collider, ColliderDensity, RigidBody};
use bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct PhysicsBundle {
    collider: Collider,
    collider_density: ColliderDensity,
    rigid_body: RigidBody,
}

const PLAYER_SIZE: f32 = 10.;

impl PhysicsBundle {
    pub fn player() -> Self {
        Self {
            collider: Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
            collider_density: ColliderDensity(0.2),
            rigid_body: RigidBody::Dynamic,
        }
    }
}