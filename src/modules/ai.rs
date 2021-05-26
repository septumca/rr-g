use core::f32;

use bevy::prelude::*;
use bevy_rapier2d::{na::{Isometry, Isometry2, Point2, Vector2}, rapier::parry::{self, query::{Ray, RayCast}}};
use rand::prelude::*;

use crate::modules::utils::get_rotated_vector;

use super::{actor, arena, ball, helpers, round, team, utils};
pub struct PlayerControlled {}
#[derive(Debug)]
pub struct AiControlled {
    pub role: Option<AiRole>
}
impl AiControlled {
    pub fn new() -> Self {
        Self {
            role: None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AiRole {
    GetBall,
    Move
}

struct AiActorData {
    entity: Entity,
    position: Vec2,
    role: Option<AiRole>,
    has_ball: bool,
}
struct PlayerActorData {
    action: actor::ActorAction,
    position: Vec2,
    has_ball: bool
}

pub fn get_closest_from_query(
    ball_position: &Vec3,
    query: &Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>
) -> ((Entity, f32), (Entity, f32)) {
    let mut closest_ai_actor: Option<(Entity, f32)> = None;
    let mut closest_player_actor: Option<(Entity, f32)> = None;

    for (entity, transform, ai_controlled, player_controller) in query.iter()  {
        let distance = Vec2::from(transform.translation).distance(Vec2::from(*ball_position));
        if ai_controlled.is_some() && (closest_ai_actor.is_none() || closest_ai_actor.unwrap().1 > distance) {
            closest_ai_actor = Some((entity, distance));
        }
        if player_controller.is_some() && (closest_player_actor.is_none() || closest_player_actor.unwrap().1 > distance) {
            closest_player_actor = Some((entity, distance));
        }
    }

    (closest_ai_actor.unwrap(), closest_player_actor.unwrap())
}

fn get_free_vector(ai_actor_position: &Vec2, player_actors: &Vec<PlayerActorData>, zone: &parry::shape::Ball, ray_direction: &Vector2<f32>) -> Option<Vec2> {
    let ray = Ray::new(Point2::new(ai_actor_position.x, ai_actor_position.y), *ray_direction);
    let blocked = player_actors.iter().any(|player_actor_data| {
        let transform = Isometry2::new(Vector2::new(player_actor_data.position.x, player_actor_data.position.y), 0.0);
        zone.intersects_ray(&transform, &ray, round::ROUND_TIME)
    });

    if blocked {
        None
    } else {
        Some(Vec2::new(ray_direction.x, ray_direction.y).normalize())
    }
}

pub fn process_ai(
    mut commands: Commands,
    helper_materials: Res<helpers::HelperMaterials>,
    mut query_actors: QuerySet<(
        Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>,
        Query<(Entity, &mut actor::Actor, &Transform, &mut AiControlled)>,
        Query<(Entity, &actor::Actor, &Transform), With<PlayerControlled>>,
    )>,
    query_ball: Query<&Transform, With<ball::Ball>>,
    query_goal_posts: QuerySet<(
        Query<&Transform, (With<arena::GoalPost>, With<AiControlled>)>,
        Query<&Transform, (With<arena::GoalPost>, With<PlayerControlled>)>,
    )>,
    ball_possession: Res<ball::BallPossession>,
    arena: Res<arena::Arena>,
) {
    let mut rng = thread_rng();
    let ball_transform = query_ball.single();

    let mut player_goalpost_position = Vec2::from(query_goal_posts.q1().single().expect("Cannot get player_goalpost!").translation);
    let mut ai_goalpost_position = Vec2::from(query_goal_posts.q0().single().expect("Cannot get player_goalpost!").translation);

    let mut ai_will_have_ball = false;
    if ball_possession.is_free() && ball_transform.is_ok() {
        let ball_position = ball_transform.unwrap().translation;

        let (
            (closest_ai_entity, closest_ai_distance),
            (_closest_player_entity, closest_player_distance)
        ) = get_closest_from_query(&ball_position, query_actors.q0());
        let closest_ai_guard_distance = closest_ai_distance - (actor::PLAYER_GUARD_RADIUS - 10.0);

        let (_entity, mut actor, transform, mut ai) = query_actors.q1_mut().get_mut(closest_ai_entity).unwrap();
        if closest_ai_distance < closest_player_distance || closest_ai_guard_distance > closest_player_distance {
            actor.queue_action(actor::ActorAction::Running { x: ball_position.x, y: ball_position.y });
            ai.role = Some(AiRole::GetBall);
            ai_will_have_ball = closest_ai_distance < closest_player_distance;
        } else if closest_ai_guard_distance < closest_player_distance {
            let ai_position = Vec2::from(transform.translation);
            let ratio = closest_ai_guard_distance / closest_ai_distance;
            let position = ((Vec2::from(ball_position) - ai_position) * ratio) + ai_position;
            actor.queue_action(actor::ActorAction::Running { x: position.x, y: position.y });
            ai.role = Some(AiRole::Move);
        }
    }

    if let Some(entity_with_ball) = ball_possession.get() {
        let (_entity, _transform, ai_controlled, _player_controlled) = query_actors.q0().get(entity_with_ball).unwrap();
        ai_will_have_ball = ai_controlled.is_some();
    }
    let player_has_ball = !ball_possession.is_free() && !ai_will_have_ball;

    //raytracing, start with straight line and gruadually deviate by some margin, find suitable vector
    //this works lot better, but need to somehow figure out how to steer actor to center of net
    //(i.e. dont start with 0 rotation but with rotation based on goal post center and then work around that)
    //instead of goal post center it can be also another target, e.g. some wing position or position near ball carrier
    let ai_actors: Vec<AiActorData> = query_actors
        .q1_mut()
        .iter_mut()
        .map(|(entity, _actor, transform, ai)| -> AiActorData {
            let mut has_ball = false;
            if let Some(entity_with_ball) = ball_possession.get() {
                has_ball = entity_with_ball == entity;
            }
            AiActorData {
                entity,
                position: Vec2::from(transform.translation),
                role: ai.role,
                has_ball,
            }
        })
        .collect();
    let player_actors: Vec<PlayerActorData> = query_actors
        .q2()
        .iter()
        .map(|(entity, actor, transform)| -> PlayerActorData {
            let mut has_ball = false;
            if let Some(entity_with_ball) = ball_possession.get() {
                has_ball = entity_with_ball == entity;
            }
            PlayerActorData {
                action: actor.act_action,
                position: Vec2::from(transform.translation),
                has_ball
            }
        })
        .collect();

    for ai_actor_data in ai_actors.iter() {
        let target_position = player_goalpost_position.clone(); //TODO: determine target location, see above
        if ai_actor_data.role.is_some() {
            continue;
        }

        let b = parry::shape::Ball::new(actor::PLAYER_GUARD_RADIUS); //sometimes ai ends in the player actor guard range regardless so add little bit leaway
        // let signum = (player_goalpost_position.x - ai_actor_data.position.x).signum();

        let mut chosen_movement: Option<Vec2> = None;
        let step = 0.1;
        let start_angle = (target_position.y - ai_actor_data.position.y).atan2(target_position.x -  ai_actor_data.position.x);
        let mut total_increment = 0.0;

        while total_increment < f32::consts::FRAC_PI_2 && chosen_movement.is_none() {
            let ray_direction = get_rotated_vector(start_angle + total_increment).normalize() * (actor::PLAYER_RUN_SPEED / round::ROUND_TIME);
            chosen_movement = get_free_vector(&ai_actor_data.position, &player_actors, &b, &ray_direction);

            if chosen_movement.is_none() && total_increment != 0.0 {
                let ray_direction = get_rotated_vector(-(start_angle + total_increment)).normalize() * (actor::PLAYER_RUN_SPEED / round::ROUND_TIME);
                chosen_movement = get_free_vector(&ai_actor_data.position, &player_actors, &b, &ray_direction);
            }
            total_increment += step;
        }

        if let Some(chm) = chosen_movement {
            //in this vector, find one where we don't go to much away from goalpost (e.g. too much up or down)

            let chm = ai_actor_data.position + (chm * actor::PLAYER_RUN_SPEED);
            if let Ok(mut actor) = query_actors.q1_mut().get_component_mut::<actor::Actor> (ai_actor_data.entity) {
                actor.set_action(actor::ActorAction::Running { x: chm.x, y: chm.y });
            }
        }
    }

    for (entity, actor, transform, _ai) in query_actors.q1_mut().iter_mut() {
        match actor.act_action {
            actor::ActorAction::Running { x, y } => {
                let he = helpers::spawn_movement_helper(
                    &mut commands,
                    &helper_materials,
                    Vec2::new(x, y),
                    Vec2::new(transform.translation.x, transform.translation.y),
                    entity.clone(),
                    helpers::HelperType::Run
                );
                commands.entity(he).insert(AiControlled::new());
            }
            _ => ()
        };
    }

    //what about throws? if we somehow determine that it would be benefical to throw then throw
    //(by comparing movements across ai actors, if there is possibility that some other ai actor would be able to move more forward, then pass the ball)
    //problem with throws is unlimited range (= implement range on throws, and/or moving idle actors to intercept ball)
    //or don't allow to score with throw (but this isn't probably good idea)
}

pub fn reset_ai_roles(
    mut query: Query<&mut AiControlled, With<actor::Actor>>,
) {
    for mut ai in query.iter_mut() {
        ai.role = None;
    }
}