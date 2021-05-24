use bevy::prelude::*;
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{RigidBodySet},
    }
};
use super::{actor, animation, collision, matchup, physics, team, utils};

pub struct Ball {}
pub struct BallThrown(Vec2);

pub struct BallPossession {
    actor: Option<Entity>,
}
impl BallPossession {
    pub fn new() -> Self {
        Self {
            actor: None,
        }
    }
    pub fn has_actor_ball(&self, actor: Entity) -> bool {
        if let Some(a) = self.actor {
            a == actor
        } else {
            false
        }
    }
    pub fn get(&self) -> Option<Entity> {
        self.actor
    }
    pub fn is_free(&self) -> bool {
        self.actor.is_none()
    }
    pub fn set(&mut self, actor: Entity) {
        self.actor = Some(actor);
    }
    pub fn clear(&mut self) {
        self.actor = None;
    }
}

pub enum BallEvent {
    Pickup { actor_entity: Entity, ball_entity: Entity },
    Drop { entity: Entity, position: Vec2, velocity_vector: Vec2 },
    Throw { entity: Entity, position: Vec2, throw_target: Vec2 },
    WallBounce { ball_entity: Entity },
}

pub struct BallTexture(Handle<TextureAtlas>);

const BALL_LINEAR_DAMPING_DROPPED: f32 = 1.5;
const BALL_LINEAR_DAMPING_BOUNCED: f32 = 0.5;

pub fn setup_ball_material(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("ball.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(8.0, 8.0), 4, 1);
    commands.insert_resource(BallTexture(texture_atlases.add(texture_atlas)));
}

pub fn add_ball_to_arena(
    mut commands: Commands,
    query_ball: Query<Entity, With<Ball>>,
    ball_sprite: Res<BallTexture>,
    matchup: Res<matchup::Matchup>
) {
    if let Ok(entity) = query_ball.single() {
        commands.entity(entity).despawn_recursive();
    }

    let position = match matchup.serving_side {
        team::Team::Home => matchup.ball_home_position,
        team::Team::Away => matchup.ball_away_position
    };
    spawn_ball(&mut commands, &ball_sprite, position, Vec2::ZERO, None);
}

pub fn spawn_ball(
    commands: &mut Commands,
    ball_sprite: &Res<BallTexture>,
    position: Vec2,
    velocity_vector: Vec2,
    throw_target: Option<Vec2>,
) {
    let linear_damping = if throw_target.is_some() { 0.0 } else { BALL_LINEAR_DAMPING_DROPPED };
    let e = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: ball_sprite.0.clone(),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, utils::PLAYING_FIELD_Z)),
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
            physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set,  None, None, Some(BALL_LINEAR_DAMPING_BOUNCED*5.0));
        }
    }
}

pub fn handle_ball_events(
    mut commands: Commands,
    mut events: EventReader<BallEvent>,
    ball_sprite: Res<BallTexture>,
    mut query_actor: Query<(&mut actor::Actor, &mut animation::Animation)>,
    query_ball: Query<&RigidBodyHandleComponent, With<Ball>>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut ball_possession: ResMut<BallPossession>,
) {
    for event in events.iter() {
        match *event {
            BallEvent::Drop { entity, position, velocity_vector} => {
                if let Ok((mut actor, mut animation)) = query_actor.get_mut(entity) {
                    actor::change_ball_possession(&mut actor, &mut animation, false);
                    ball_possession.clear();
                }
                let norm_vel = velocity_vector.normalize();
                let ball_position = Vec2::new(
                    position.x + norm_vel.x*(utils::TRUE_SPRITE_SIZE/2.0),
                    position.y + norm_vel.y*(utils::TRUE_SPRITE_SIZE/2.0),
                );
                let ball_velocity = Vec2::new(velocity_vector.x, velocity_vector.y) * 1.5;
                spawn_ball(&mut commands, &ball_sprite, ball_position, ball_velocity, None);
            },
            BallEvent::Throw { entity, position, throw_target} => {
                if let Ok((mut actor, mut animation)) = query_actor.get_mut(entity) {
                    actor::change_ball_possession(&mut actor, &mut animation, false);
                    ball_possession.clear();
                }
                let delta = (throw_target - position).normalize();
                let ball_position = Vec2::new(
                    position.x + delta.x*utils::TRUE_SPRITE_SIZE,
                    position.y + delta.y*utils::TRUE_SPRITE_SIZE,
                );
                let ball_velocity = Vec2::new(delta.x, delta.y) * 300.0; //TODO: replace with throw power
                spawn_ball(&mut commands, &ball_sprite, ball_position, ball_velocity, Some(throw_target));
            },
            BallEvent::Pickup { actor_entity, ball_entity} => {
                if let Ok((mut actor, mut animation)) = query_actor.get_mut(actor_entity) {
                    actor::change_ball_possession(&mut actor, &mut animation, true);
                    ball_possession.set(actor_entity);
                }
                commands.entity(ball_entity).despawn();
            },
            BallEvent::WallBounce { ball_entity } => {
                if let Ok(rigid_body_handle) = query_ball.get(ball_entity) {
                    physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set,  None, None, Some(BALL_LINEAR_DAMPING_BOUNCED));
                }
            }
        }
    }
}