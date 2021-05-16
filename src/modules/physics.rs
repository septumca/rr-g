use bevy::prelude::*;
use bevy_rapier2d::{
    na::Vector2,
    physics::{RigidBodyHandleComponent, RapierConfiguration},
    rapier::{
        dynamics::{RigidBodySet, RigidBodyBuilder},
        geometry::{ColliderBuilder},
    }
};

pub fn set_velocity(
    rigid_body_handle: &RigidBodyHandleComponent,
    rigid_body_set: &mut ResMut<RigidBodySet>,
    velocity: Vec2,
) {
    if let Some(rigid_body) = rigid_body_set.get_mut(rigid_body_handle.handle()) {
        rigid_body.set_linvel(Vector2::new(velocity.x, velocity.y), true);
    }
}

pub fn create_physics_player(
    commands: &mut Commands,
    e: Entity,
    position: Vec2
) {
    commands.entity(e).insert(
    RigidBodyBuilder::new_dynamic()
        .translation(position.x, position.y)
        .lock_rotations()
    );
    commands.entity(e).insert(
    ColliderBuilder::capsule_y(4.0, 10.0)
        .density(80.0)
        .friction(0.0)
        .restitution(0.2)
        .user_data(e.to_bits() as u128)
    );
}

pub fn pause_physics(
    mut physics_state: ResMut<RapierConfiguration>,
) {
    physics_state.physics_pipeline_active = false;
}

pub fn resume_physics(
    mut physics_state: ResMut<RapierConfiguration>,
) {
    physics_state.physics_pipeline_active = true;
}

pub fn create_physics_ball(
    commands: &mut Commands,
    e: Entity,
    position: Vec2,
    velocity_vector: Vec2,
    linear_damping: f32,
) {
    commands.entity(e).insert(
    RigidBodyBuilder::new_dynamic()
        .translation(position.x, position.y)
        .linvel(velocity_vector.x, velocity_vector.y)
        .linear_damping(linear_damping)
        .lock_rotations()
    );
    commands.entity(e).insert(
    ColliderBuilder::capsule_x(4.0, 3.0)
        .density(1.0)
        .friction(0.7)
        .restitution(0.5)
        .user_data(e.to_bits() as u128)
    );
}