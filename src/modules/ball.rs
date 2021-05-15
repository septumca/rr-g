use bevy::prelude::*;

pub struct Ball {}
pub struct BallMaterial(Handle<ColorMaterial>);

pub fn setup_ball_material(commands: &mut Commands, asset_server: &Res<AssetServer>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(BallMaterial(materials.add(asset_server.load("ball.png").into())));
}

pub fn spawn_ball(
    commands: &mut Commands,
    ball_material: &Res<BallMaterial>,
    position: Vec2,
    velocity_vector: Vec2,
) {
    let e = commands
        .spawn_bundle(SpriteBundle {
            material: ball_material.0.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 6.0)),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 2.0)),
            ..Default::default()
        })
        .insert(Ball {})
        .insert(super::collision::ColliderType::Ball)
        .id();

    super::physics::create_physics_ball(commands, e, position, velocity_vector);
}