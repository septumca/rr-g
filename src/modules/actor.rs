use std::time::Duration;
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
    ball,
    helpers,
};

const PLAYER_SPEED: f32 = 100.0;
pub struct ActorTextures {
    red: Handle<TextureAtlas>,
    blue: Handle<TextureAtlas>,
}
pub struct ActionTimer(Timer);

pub struct BallPossession(pub bool);

pub struct CurrentControlMode(pub ControlMode);
#[derive(Debug)]
pub enum ControlMode {
    Run,
    Throw
}


#[derive(Clone, Copy, Debug)]
pub enum ActorAction {
    Idle,
    Running { x: f32, y: f32 },
    Throwing { x: f32, y: f32 },
    // Diving,
    Recovering(f32)
}
pub struct Actor {
    pub act_action: ActorAction,
    queued_action: Option<ActorAction>,
}
impl Actor {
    pub fn new_idle() -> Self {
        Self {
            act_action: ActorAction::Idle,
            queued_action: None
        }
    }
    pub fn trigger_queued_action(&mut self) {
        self.act_action = self.queued_action.unwrap_or(ActorAction::Idle);
    }
    pub fn set_action(&mut self, action: ActorAction) {
        self.act_action = action;
        self.queued_action = None;
    }
    pub fn queue_action(&mut self, action: ActorAction) {
        match self.act_action {
            ActorAction::Idle => {
                self.set_action(action);
            },
            _ => {
                self.queued_action = Some(action);
            }
        }
    }
}

pub struct Selected {}

fn get_running_indexes(ball_possession: bool) -> Vec<usize> {
    if ball_possession  { vec![3, 4, 3, 5] } else { vec![0, 1, 0, 2] }
}

fn get_idle_indexes(ball_possession: bool) -> Vec<usize> {
    if ball_possession { vec![3] } else { vec![0] }
}


pub fn setup_actor_sprites(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("actors-red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 14, 1);
    let texture_atlas_handle_red = texture_atlases.add(texture_atlas);
    let texture_handle = asset_server.load("actors-blue.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 14, 1);
    let texture_atlas_handle_blue = texture_atlases.add(texture_atlas);

    commands.insert_resource(ActorTextures{
        red: texture_atlas_handle_red,
        blue: texture_atlas_handle_blue
    });
}


pub fn spawn_actor(commands: &mut Commands, actor_sprites: &Res<ActorTextures>, position: Vec2, blue: bool, flipped: bool) {
    let e = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: if blue { actor_sprites.blue.clone() } else { actor_sprites.red.clone() },
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 1.0)),
            sprite: TextureAtlasSprite {
                flip_x: flipped,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Actor::new_idle())
        .insert(BallPossession(false))
        .insert(animation::Animation::new(vec![0]))
        .insert(animation::AnimationTimer(Timer::from_seconds(1.0/8.0, true)))
        .insert(ActionTimer(Timer::from_seconds(1.0, false)))
        .insert(collision::ColliderType::Actor)
        .id();
    physics::create_physics_actor(commands, e, position);
}

pub fn reset_control_mode(
    mut control_mode: ResMut<CurrentControlMode>,
) {
    control_mode.0 = ControlMode::Run;
}

pub fn reset_move_actions(
    mut query: Query<&mut Actor>,
) {
    for mut actor in query.iter_mut() {
        match actor.act_action {
            ActorAction::Running { x: _, y: _ } => {
                actor.set_action(ActorAction::Idle);
            },
            _ => ()
        }
    }
}

pub fn reset_action_timer(timer: &mut ActionTimer, t: f32) {
    timer.0.set_duration(Duration::from_secs_f32(t));
    timer.0.set_repeating(false);
    timer.0.reset();
}


pub fn handle_actors_action_finish(
    time: Res<Time>,
    mut ball_events: EventWriter<ball::BallEvent>,
    mut query: Query<(Entity, &mut Actor, &mut Transform, &mut ActionTimer, &animation::Animation, &mut BallPossession)>,
) {
    for (
        entity,
        mut actor,
        transform,
        mut timer,
        animation,
        ball_possession
    ) in query.iter_mut() {
        let is_action_finished = match actor.act_action {
            ActorAction::Idle => false,
            ActorAction::Running { x, y} => {
                let d_x = transform.translation.x - x;
                let d_y = transform.translation.y - y;

                d_x.abs() < 1.0 && d_y.abs() < 1.0
            },
            ActorAction::Throwing { x, y } => {
                if animation.finished {
                    if ball_possession.0 {
                        ball_events.send(ball::BallEvent::Throw {
                            entity,
                            position: Vec2::new(transform.translation.x, transform.translation.y),
                            throw_target: Vec2::new(x, y),
                        });
                    } else {
                        println!("Wanted to throw non-existing ball!");
                    }
                }
                animation.finished
            },
            ActorAction::Recovering(_) => {
                timer.0.tick(time.delta());
                timer.0.finished()
            }
        };
        if is_action_finished {
            reset_action_timer(&mut timer, 1.0);
            actor.trigger_queued_action();
        }
    }
}

pub fn handle_actor_action_start(
    mut ball_events: EventWriter<ball::BallEvent>,
    mut query: Query<(
        Entity,
        &Actor,
        &Transform,
        &RigidBodyHandleComponent,
        &mut TextureAtlasSprite,
        &mut animation::Animation,
        &mut ActionTimer,
        &BallPossession,
    ), Changed<Actor>>,
    mut rigid_body_set: ResMut<RigidBodySet>,
) {
    for (
        entity,
        actor,
        transform,
        rigid_body_handle,
        mut sprite,
        mut animation,
        mut timer,
        ball_possession
    ) in query.iter_mut() {
        match actor.act_action {
            ActorAction::Idle => {
                animation.update_sprites_indexes(get_idle_indexes(ball_possession.0), true);
                physics::set_velocity(rigid_body_handle, &mut rigid_body_set,  Vec2::ZERO);
            },
            ActorAction::Running { x, y} => {
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize() * PLAYER_SPEED;
                sprite.flip_x = delta.x < 0.0;
                physics::set_velocity(rigid_body_handle, &mut rigid_body_set, Vec2::new(delta.x, delta.y));
                animation.update_sprites_indexes(get_running_indexes(ball_possession.0), true);
            },
            ActorAction::Throwing { x, y} => {
                animation.update_sprites_indexes(vec![7, 8, 9], false);
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize();
                sprite.flip_x = delta.x < 0.0;
            }
            ActorAction::Recovering(t) => {
                animation.update_sprites_indexes(vec![6], true);
                reset_action_timer(&mut timer, t);
                if ball_possession.0 {
                    let rb_vel = rigid_body_set.get_mut(rigid_body_handle.handle()).and_then(|rb| Some(rb.linvel()));
                    if rb_vel.is_none() {
                        return;
                    }
                    let rb_vel = rb_vel.unwrap();
                    ball_events.send(ball::BallEvent::Drop {
                        entity,
                        position: Vec2::new(transform.translation.x, transform.translation.y),
                        velocity_vector: Vec2::new(rb_vel.x, rb_vel.y),
                    });
                }
            }
        };
    }
}


//TODO this can be done better - use children entity and display position dynamically based on frame?
pub fn update_sprite(
    mut query: Query<(&Actor, &mut animation::Animation, &BallPossession)>
) {
    for (actor, mut animation, ball_possession) in query.iter_mut() {
        let indexes = match actor.act_action {
            ActorAction::Idle => {
                Some(get_idle_indexes(ball_possession.0))

            },
            ActorAction::Running { x: _, y: _ } => {
                Some(get_running_indexes(ball_possession.0))
            },
            _ => None
        };

        if let Some(i) = indexes {
            animation.set_sprite_indexes(i);
        }
    }
}

pub fn update_helpers(
    mut commands: Commands,
    mut query: Query<(Entity, &Actor)>,
    query_movement_helper: Query<(Entity, &helpers::MovementHelper)>
) {
    for (entity, actor) in query.iter_mut() {
        let should_keep_helpers = match actor.act_action {
            ActorAction::Running { x: _, y: _ } => true,
            _ => false,
        };

        if should_keep_helpers {
            continue;
        }

        for (helper_entity, actor_entity) in query_movement_helper.iter() {
            if actor_entity.actor == entity {
                commands.entity(helper_entity).despawn_recursive();
            }
        }
    }
}
