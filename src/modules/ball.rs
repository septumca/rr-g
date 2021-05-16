use bevy::prelude::*;
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{RigidBodySet},
    }
};
use super::{
    animation,
    physics,
    collision,
};
pub struct Ball {}
pub struct BallThrown(Vec2);

pub enum BallEvent {
    Drop { position: Vec2, velocity_vector: Vec2 },
    Throw { position: Vec2, throw_target: Vec2 },
}

pub struct BallTexture(Handle<TextureAtlas>);

pub fn setup_ball_material(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("ball.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(8.0, 8.0), 4, 1);
    commands.insert_resource(BallTexture(texture_atlases.add(texture_atlas)));
}

pub fn spawn_ball(
    commands: &mut Commands,
    ball_sprite: &Res<BallTexture>,
    position: Vec2,
    velocity_vector: Vec2,
    throw_target: Option<Vec2>,
) {
    let linear_damping = if throw_target.is_some() { 0.0 } else { 1.5 };
    let e = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: ball_sprite.0.clone(),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 2.0)),
            ..Default::default()
        })
        .insert(Ball {})
        .insert(animation::Animation::new(vec![0]))
        .insert(animation::AnimationTimer(Timer::from_seconds(1.0/8.0, true)))
        .insert(collision::ColliderType::Ball)
        .id();

    if throw_target.is_some() {
        commands.entity(e).insert(BallThrown(throw_target.unwrap()));
    }

    physics::create_physics_ball(commands, e, position, velocity_vector, linear_damping);
}

pub fn update_thrown_ball(
    query: Query<(&Transform, &BallThrown, &RigidBodyHandleComponent), With<Ball>>,
    mut rigid_body_set: ResMut<RigidBodySet>
) {
    for (transform, ball_thrown, rigid_body_handle) in query.iter() {
        let d_x = transform.translation.x - ball_thrown.0.x;
        let d_y = transform.translation.y - ball_thrown.0.y;
        if d_x.abs() < 10.0 && d_y.abs() < 10.0 {
            physics::set_velocity(rigid_body_handle, &mut rigid_body_set,  Vec2::ZERO);
        }
    }
}

pub fn handle_ball_events(
    mut commands: Commands,
    mut events: EventReader<BallEvent>,
    ball_sprite: Res<BallTexture>,
) {
    for event in events.iter() {
        match *event {
            BallEvent::Drop { position, velocity_vector} => {
                let norm_vel = velocity_vector.normalize();
                let ball_position = Vec2::new(
                    position.x + norm_vel.x*16.0,
                    position.y + norm_vel.y*16.0,
                );
                let ball_velocity = Vec2::new(velocity_vector.x, velocity_vector.y) * 1.5;
                spawn_ball(&mut commands, &ball_sprite, ball_position, ball_velocity, None);
            },
            BallEvent::Throw { position, throw_target} => {
                let delta = (throw_target - position).normalize();
                let ball_position = Vec2::new(
                    position.x + delta.x*32.0,
                    position.y + delta.y*32.0,
                );
                let ball_velocity = Vec2::new(delta.x, delta.y) * 300.0; //TODO: replace with throw power
                spawn_ball(&mut commands, &ball_sprite, ball_position, ball_velocity, Some(throw_target));
            },
        }
    }
}