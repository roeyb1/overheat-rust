use avian3d::prelude::{CoefficientCombine, Collider, ExternalForce, ExternalImpulse, Friction, LinearVelocity, LockedAxes, Mass, Position, RigidBody};
use bevy::{ecs::query::QueryData, prelude::{Bundle, Entity}};

#[derive(Bundle)]
pub struct PhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    external_forces: ExternalForce,
    external_impulse: ExternalImpulse,
    lock_axes: LockedAxes,
    friction: Friction,
}

const PLAYER_SIZE: f32 = 0.5;

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct CharacterQuery {
    pub external_force: &'static mut ExternalForce,
    pub external_impulse: &'static mut ExternalImpulse,
    pub linear_velocity: &'static LinearVelocity,
    pub mass: &'static Mass,
    pub position: &'static Position,
    pub entity: Entity,
}

impl PhysicsBundle {
    pub fn player() -> Self {
        Self {
            collider: Collider::capsule(PLAYER_SIZE, PLAYER_SIZE),
            rigid_body: RigidBody::Dynamic,
            external_forces: ExternalForce::ZERO.with_persistence(false),
            external_impulse: ExternalImpulse::ZERO.with_persistence(false),
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z()
                .lock_translation_y(),
            friction: Friction::new(0.).with_combine_rule(CoefficientCombine::Min),
        }
    }
}