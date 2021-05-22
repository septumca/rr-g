use bevy::prelude::*;
use bevy_rapier2d::{
    na::Vector2,
    physics::{RigidBodyHandleComponent, RapierConfiguration},
    rapier::{
        dynamics::{RigidBodySet, RigidBodyBuilder},
        geometry::{ColliderBuilder},
        math::Isometry,
    }
};

pub fn set_rb_properties(
    rigid_body_handle: &RigidBodyHandleComponent,
    rigid_body_set: &mut ResMut<RigidBodySet>,
    velocity: Option<Vec2>,
    position: Option<Vec2>,
    linear_damping: Option<f32>,
) {
    if let Some(rigid_body) = rigid_body_set.get_mut(rigid_body_handle.handle()) {
        if let Some(v) = velocity {
            rigid_body.set_linvel(Vector2::new(v.x, v.y), true);
        }
        if let Some(ld) = linear_damping {
            rigid_body.linear_damping = ld;
        }
        if let Some(p) = position {
            rigid_body.set_position(Isometry::translation(p.x, p.y), true);
        }
    }
}

pub fn get_velocity(
    rigid_body_handle: &RigidBodyHandleComponent,
    rigid_body_set: &ResMut<RigidBodySet>,
) -> Option<Vec2> {
    rigid_body_set
        .get(rigid_body_handle.handle())
        .and_then(|rb| {
            let rb_vel = rb.linvel();
            Some(Vec2::new(rb_vel.x, rb_vel.y))
        })
}

pub fn create_physics_actor(
    commands: &mut Commands,
    e: Entity,
    position: Vec2,
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

pub fn create_physics_wall(
    commands: &mut Commands,
    e: Entity,
    position: Vec2,
    w: f32,
    h: f32,
) {
    commands.entity(e).insert(
    RigidBodyBuilder::new_static()
        .translation(position.x, position.y)
        .lock_rotations()
    );
    commands.entity(e).insert(
    ColliderBuilder::cuboid(w/2.0, h/2.0)
        .density(1.0)
        .friction(0.7)
        .restitution(0.1)
        .user_data(e.to_bits() as u128)
    );
}

pub fn create_physics_goalpost(
    commands: &mut Commands,
    e: Entity,
    position: Vec2,
    w: f32,
    h: f32,
) {
    commands.entity(e).insert(
    RigidBodyBuilder::new_static()
        .translation(position.x, position.y)
        .lock_rotations()
    );
    commands.entity(e).insert(
    ColliderBuilder::cuboid(w/2.0, h/2.0)
    .density(1.0)
    .friction(0.7)
    .restitution(0.1)
        .sensor(true)
        .user_data(e.to_bits() as u128)
    );
}