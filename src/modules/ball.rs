use bevy::prelude::*;
use heron::prelude::*;

pub struct Ball {}
pub struct BallMaterial(Handle<ColorMaterial>);

pub fn setup_ball_material(commands: &mut Commands, asset_server: &Res<AssetServer>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(BallMaterial(materials.add(asset_server.load("ball.png").into())));
}

pub fn spawn_ball(
    commands: &mut Commands,
    ball_material: &Res<BallMaterial>,
    position: Vec3
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: ball_material.0.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 6.0)),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 2.0)),
            ..Default::default()
        })
        .insert(Ball {})
        .insert(super::collision::ColliderType::Ball)
        .insert(Body::Capsule { half_segment: -4.0, radius: 5.0 })
        .insert(RotationConstraints::lock())
        .insert(PhysicMaterial {
            restitution: 1.0, // Define the restitution. Higher value means more "bouncy"
            density: 0.5, // Define the density. Higher value means heavier.
            friction: 1.0, // Define the friction. Higher value means higher friction.
        })
        .insert(RotationConstraints::lock())
        .insert(Velocity::from(Vec2::ZERO));
}