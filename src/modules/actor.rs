use std::time::Duration;
use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodySet},
    }
};
use super::{animation, ai, ball, collision, helpers, physics, team, utils};

pub const PLAYER_RUN_SPEED: f32 = 100.0;
const PLAYER_TACKLE_SPEED: f32 = 225.0;
pub const PLAYER_GUARD_RADIUS: f32 = 60.0;
pub const PLAYER_TACKLE_RADIUS: f32 = 120.0;
const PLAYER_RECOVERY_TIME_BUMPED: f32 = 0.3;
const PLAYER_RECOVERY_TIME_TACKLED: f32 = 0.9;
const PLAYER_RECOVERY_LINEAR_DAMPING: f32 = 1.5;
pub const PLAYER_THROWING_POWER: f32 = 0.5;

pub struct ActorTextures {
    red: Handle<TextureAtlas>,
    blue: Handle<TextureAtlas>,
}
pub struct ActionTimer(Timer);
// pub struct BallPossession(pub bool);
pub struct IsTackleTarget(pub bool);
pub struct CurrentControlMode(pub ControlMode);
#[derive(Debug)]
pub enum ControlMode {
    Run,
    Throw
}

pub enum ActorEvents{
    LookForTackle {
        entity: Entity,
        team: team::Team,
        position: Vec2
    },
    ActorsCollided {
        actor_entity: Entity,
        actor_action: ActorAction,
        other_actor_entity: Entity,
        other_actor_action: ActorAction
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActorAction {
    Idle,
    Lookout,
    Running { x: f32, y: f32 },
    Throwing { x: f32, y: f32 },
    Tackling { x: f32, y: f32 },
    Recovering(f32)
}
#[derive(Debug)]
pub struct Actor {
    pub act_action: ActorAction,
    queued_action: Option<ActorAction>,
    has_tackled: bool,
}
impl Actor {
    pub fn new() -> Self {
        Self {
            act_action: ActorAction::Lookout,
            queued_action: None,
            has_tackled: false,
        }
    }
    pub fn trigger_queued_action(&mut self, has_ball: bool) {
        self.act_action = self.queued_action.unwrap_or(if self.has_tackled || has_ball { ActorAction::Idle } else { ActorAction::Lookout });
    }
    pub fn set_action(&mut self, action: ActorAction) {
        self.act_action = action;
        self.queued_action = None;
    }
    pub fn queue_action(&mut self, action: ActorAction) {
        match self.act_action {
            ActorAction::Idle | ActorAction::Lookout => {
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
    let texture_handle = asset_server.load("players-red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(utils::ACTOR_SPRITE_SIZE_W_PADDING, utils::ACTOR_SPRITE_SIZE_W_PADDING), 13, 1);
    let texture_atlas_handle_red = texture_atlases.add(texture_atlas);
    let texture_handle = asset_server.load("players-blue.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(utils::ACTOR_SPRITE_SIZE_W_PADDING, utils::ACTOR_SPRITE_SIZE_W_PADDING), 13, 1);
    let texture_atlas_handle_blue = texture_atlases.add(texture_atlas);

    commands.insert_resource(ActorTextures{
        red: texture_atlas_handle_red,
        blue: texture_atlas_handle_blue
    });
}


pub fn spawn_actor(
    commands: &mut Commands,
    actor_sprites: &Res<ActorTextures>,
    position: Vec2,
    team: team::Team,
    is_player_controlled: bool,
) -> Entity {
    let (texture_atlas, is_left_side) = match team {
        team::Team::Home => (actor_sprites.blue.clone(), false),
        team::Team::Away => (actor_sprites.red.clone(), true),
    };

    let e = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas,
            transform: Transform::from_translation(
                Vec3::new(
                    position.x,
                    position.y,
                    utils::PLAYING_FIELD_Z
                )
            ),
            sprite: TextureAtlasSprite {
                flip_x: is_left_side,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Actor::new())
        .insert(team)
        .insert(IsTackleTarget(false))
        .insert(animation::Animation::new(vec![0]))
        .insert(animation::AnimationTimer(Timer::from_seconds(1.0/8.0, true)))
        .insert(ActionTimer(Timer::from_seconds(1.0, false)))
        .insert(collision::ColliderType::Actor)
        .id();

    if is_player_controlled {
        commands.entity(e).insert(ai::PlayerControlled {});
    } else {
        commands.entity(e).insert(ai::AiControlled::new(ai::AiFocus::GuardBallCarrier, ai::AiFocus::DefendGoalPost));
    }
    physics::create_physics_actor(commands, e, position);

    e
}

pub fn reset_control_mode(
    mut control_mode: ResMut<CurrentControlMode>,
) {
    control_mode.0 = ControlMode::Run;
}

pub fn after_round_reset(
    mut query: Query<(Entity, &mut Actor, &mut IsTackleTarget)>,
    ball_possession: Res<ball::BallPossession>,
) {
    for (entity, mut actor, mut is_tackle_target) in query.iter_mut() {
        match actor.act_action {
            //actor will reset action only if running - if he was running at end of the turn he can tackle next round
            ActorAction::Running { x: _, y: _ } => {
                let has_ball = ball_possession.has_actor_ball(entity);
                actor.set_action(if has_ball { ActorAction::Idle } else { ActorAction::Lookout });
            },
            _ => ()
        }
        actor.has_tackled = false;
        is_tackle_target.0 = false;
    }
}

pub fn reset_action_timer(timer: &mut ActionTimer, t: f32) {
    timer.0.set_duration(Duration::from_secs_f32(t));
    timer.0.set_repeating(false);
    timer.0.reset();
}


pub fn handle_actors_refresh_action(
    time: Res<Time>,
    mut ball_events: EventWriter<ball::BallEvent>,
    mut query: Query<(Entity, &team::Team, &mut Actor, &mut Transform, &mut ActionTimer, &animation::Animation)>,
    mut event_tackle_target: EventWriter<ActorEvents>,
    ball_possession: Res<ball::BallPossession>,
) {
    for (
        entity,
        team,
        mut actor,
        transform,
        mut timer,
        animation,
    ) in query.iter_mut() {
        let has_ball = ball_possession.has_actor_ball(entity);
        let is_action_finished = match actor.act_action {
            ActorAction::Lookout => {
                event_tackle_target.send(ActorEvents::LookForTackle {
                    entity,
                    team: *team,
                    position: Vec2::new(transform.translation.x, transform.translation.y)
                });
                false
            },
            ActorAction::Idle => false,
            ActorAction::Running { x, y} | ActorAction::Tackling { x, y } => {
                let d_x = transform.translation.x - x;
                let d_y = transform.translation.y - y;
                //when this value is too big (e.g. 10.0), then actor get from tackle action too soon into idle
                //and it can result into tackle->idle and then colliding with running actor
                //test with from running from -100.0, 100.0 to 132.0, 48.0 and tackling actor standing at 100, 100
                d_x.abs() < 2.0 && d_y.abs() < 2.0
            },
            ActorAction::Throwing { x, y } => {
                if animation.finished {
                    if has_ball {
                        ball_events.send(ball::BallEvent::Throw {
                            entity,
                            position: Vec2::new(transform.translation.x, transform.translation.y),
                            throw_target: Vec2::new(x, y),
                            power: PLAYER_THROWING_POWER,
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
            actor.trigger_queued_action(has_ball);
        }
    }
}

pub fn handle_actor_action_start(
    mut query: Query<(
        Entity,
        &mut Actor,
        &Transform,
        &RigidBodyHandleComponent,
        &mut TextureAtlasSprite,
        &mut animation::Animation,
        &mut ActionTimer,
    ), Changed<Actor>>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    ball_possession: Res<ball::BallPossession>,
) {
    for (
        entity,
        mut actor,
        transform,
        rigid_body_handle,
        mut sprite,
        mut animation,
        mut timer,
    ) in query.iter_mut() {
        let has_ball = ball_possession.has_actor_ball(entity);
        match actor.act_action {
            ActorAction::Lookout | ActorAction::Idle => {
                animation.update_sprites_indexes(get_idle_indexes(has_ball), true);
                physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set,  Some(Vec2::ZERO), None, Some(0.0));
            },
            ActorAction::Tackling {x, y} => {
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize() * PLAYER_TACKLE_SPEED ;
                sprite.flip_x = delta.x < 0.0;
                animation.update_sprites_indexes(vec![10, 11, 12], false);
                physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set, Some(Vec2::new(delta.x, delta.y)), None, Some(0.0));
                actor.has_tackled = true;
            }
            ActorAction::Running { x, y} => {
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize() * PLAYER_RUN_SPEED;
                sprite.flip_x = delta.x < 0.0;
                animation.update_sprites_indexes(get_running_indexes(has_ball), true);
                physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set, Some(Vec2::new(delta.x, delta.y)), None, Some(0.0));
            },
            ActorAction::Throwing { x, y} => {
                animation.update_sprites_indexes(vec![7, 8, 9], false);
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize();
                sprite.flip_x = delta.x < 0.0;
                physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set, None, None, Some(0.0));
            }
            ActorAction::Recovering(t) => {
                animation.update_sprites_indexes(vec![6], true);
                reset_action_timer(&mut timer, t);
                physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set, None, None, Some(PLAYER_RECOVERY_LINEAR_DAMPING));
            }
        };
    }
}

fn get_tackle_hit_position(target_position: Vec2, target_velocity: Vec2, origin_position: Vec2) -> Option<Vec2> {
    let mut last_magnitude = f32::INFINITY;
    let mut step = 0.2;
    let actor_speed_squared = PLAYER_TACKLE_SPEED.powi(2);

    //P1 + V1*step = P2 + V2*step ---> iterate through steps, solve for V2
    loop {
        let tackle_velocity = (target_position - origin_position + (target_velocity*step))/step; //calculate needed velocity for time step
        let new_magnitude = tackle_velocity.length_squared();
        let hit_position = origin_position + (tackle_velocity*step);
        //if it's possible for actor to reach this velocity and if hit position is in tackle range then return it
        if new_magnitude < actor_speed_squared && (hit_position - origin_position).length_squared() < PLAYER_TACKLE_RADIUS.powi(2) {
            return Some(hit_position);
        }
        //if the new magnitude is higher, e.g. target is getting away and there is no chance to catch it + limit number of calculation to prevent infinite loops
        if new_magnitude > last_magnitude || step > 100.0 {
            return None;
        }
        last_magnitude = new_magnitude;
        step = step + 0.2;
    }
}

pub fn handle_actor_events(
    mut events: EventReader<ActorEvents>,
    mut ball_events: EventWriter<ball::BallEvent>,
    mut query: Query<(
        &mut Actor,
        &team::Team,
        &mut IsTackleTarget,
        &Transform,
        &RigidBodyHandleComponent
    )>,
    mut rigid_body_set: ResMut<RigidBodySet>,
    mut ball_possession: ResMut<ball::BallPossession>,
) {
    for event in events.iter() {
        match event {
            ActorEvents::ActorsCollided { actor_entity, actor_action, other_actor_entity,  other_actor_action} => {
                let recovery_time = match *other_actor_action {
                    ActorAction::Tackling { x: _, y: _ } => PLAYER_RECOVERY_TIME_TACKLED,
                    ActorAction::Running { x: _, y: _ } => {
                        match *actor_action {
                            ActorAction::Tackling  { x: _, y: _ } => 0.0,
                            _ => PLAYER_RECOVERY_TIME_BUMPED
                        }
                    },
                    _ => 0.0
                };
                let (
                    mut actor,
                    _team,
                    _is_tackle_target,
                    transform,
                    rigid_body_handle
                ) = query.get_mut(*actor_entity).expect("Cannot get actor that was hit!");

                let action = if recovery_time > 0.0 { ActorAction::Recovering(recovery_time) } else { ActorAction::Idle };
                actor.set_action(action);

                if ball_possession.has_actor_ball(*actor_entity){
                    match  *other_actor_action {
                        ActorAction::Tackling { x:_, y:_ } => {
                            ball_possession.set(*other_actor_entity);
                        },
                        _ => {
                            let rb_vel = physics::get_velocity(rigid_body_handle, &mut rigid_body_set).expect("Cannot get velocity information from actor");
                            ball_events.send(ball::BallEvent::Drop {
                                entity: *actor_entity,
                                position: Vec2::new(transform.translation.x, transform.translation.y),
                                velocity_vector: rb_vel,
                            });
                        }
                    }
                }
            },
            ActorEvents::LookForTackle { entity, position, team } => {
                let actor_tackle_radius_squared = PLAYER_GUARD_RADIUS.powi(2);
                let mut hit_position = None;
                for (
                    actor,
                    team_target,
                    mut is_tackle_target,
                    transform,
                    rigid_body_handle
                ) in query.iter_mut() {
                    let target_position = Vec2::new(transform.translation.x, transform.translation.y);
                    //TODO: maybe even when throwing, altough it seems to be bugged atm
                    let is_in_nontacklable_state = match actor.act_action {
                        ActorAction::Running { x: _, y: _ } => false,
                        _ => true
                    };
                    if team_target == team || (target_position - *position).length_squared() > actor_tackle_radius_squared || is_in_nontacklable_state || is_tackle_target.0 {
                        continue;
                    }
                    let target_velocity = physics::get_velocity(rigid_body_handle, &mut rigid_body_set).expect("Cannot get velocity information from actor");
                    hit_position = get_tackle_hit_position(target_position, target_velocity, *position);
                    if hit_position.is_some() {
                        is_tackle_target.0 = true;
                        break;
                    }
                }

                if let Some(hp) = hit_position {
                    let (
                        mut actor,
                        _team,
                        _is_tackle_target,
                        _transform,
                        _rigid_body_handle
                    ) =  query.get_mut(*entity).expect("Player that spawned LookForTackle event no longer exists!");
                    actor.set_action(ActorAction::Tackling { x: hp.x, y: hp. y });
                    actor.queue_action(ActorAction::Idle);
                }
            }
        }
    }
}

pub fn change_ball_possession(
    actor: &mut Actor,
    animation: &mut animation::Animation,
    ball_possession: bool,
) {
    let indexes = match actor.act_action {
        ActorAction::Idle => {
            Some(get_idle_indexes(ball_possession))
        },
        ActorAction::Lookout => {
            if ball_possession {
                actor.set_action(ActorAction::Idle);
            }
            Some(get_idle_indexes(ball_possession))
        }
        ActorAction::Running { x: _, y: _ } => {
            Some(get_running_indexes(ball_possession))
        },
        _ => None
    };

    if let Some(i) = indexes {
        animation.set_sprite_indexes(i);
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
