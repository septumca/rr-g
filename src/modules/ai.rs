use bevy::prelude::*;
use rand::prelude::*;

use super::{
    actor,
    arena,
    ball,
    //helpers,
    utils};
pub struct PlayerControlled {}
#[derive(Debug)]
pub struct AiControlled {
    role: Option<AiRole>
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

type AiActorData = (Entity, Vec2, Option<AiRole>, f32);
type AiPlayerData = (actor::ActorAction, Vec2, bool);


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


fn get_lane_bounds(lane_num: f32, lane_height: f32) -> (f32, f32) {
    (lane_height * lane_num - utils::WIN_H/2.0, lane_height * (lane_num + 1.0) - utils::WIN_H/2.0)
}

pub fn process_ai(
    // mut commands: Commands,
    // helper_materials: Res<helpers::HelperMaterials>,
    mut query_actors: QuerySet<(
        Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>,
        Query<(Entity, &mut actor::Actor, &Transform, &mut AiControlled)>,
        Query<(&actor::Actor, &Transform), With<PlayerControlled>>,
    )>,
    query_ball: Query<&Transform, With<ball::Ball>>,
    ball_possession: Res<ball::BallPossession>,
    arena: Res<arena::Arena>,
) {
    let mut rng = thread_rng();
    let ball_transform = query_ball.single();
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

    //get number of ai actors that don't have assigned role
    //sort them by their y-axis position/(or role)
    //split areana into <number of unassigned ai actors> lanes
    //assign lane to each ai actor
    //move actor to/in this lane
    //TODO: this feels little bit braindead, but at least it's something ¯\_( ͡° ͜ʖ ͡°)_/¯
    let mut ai_actors_vec: Vec<AiActorData> = query_actors
        .q1_mut()
        .iter_mut()
        .map(|a| -> AiActorData {
            (a.0.clone(), Vec2::from(a.2.translation), a.3.role, -1.0)
        })
        .collect();
    //this needs to be at spearate line and cannot be chained directly after collect - because sort_by does sort in place
    ai_actors_vec.sort_by(|a, b| a.1.y.partial_cmp(&b.1.y).unwrap_or(std::cmp::Ordering::Equal));
    let ai_actors_vec = ai_actors_vec
        .iter_mut()
        .enumerate()
        .map(|(i, a)| {
            (a.0, a.1, a.2, i as f32)
        });
    let lane_height = arena.height / ai_actors_vec.len() as f32;

    let ai_actors_data: Vec<(AiActorData, Vec<AiPlayerData>)> = ai_actors_vec
        .into_iter()
        .map(|a| -> (AiActorData, Vec<AiPlayerData>) {
            let lane_num = a.3;

            let (y_min, y_max) = get_lane_bounds(lane_num, lane_height);
            let player_data = query_actors
                .q2()
                .iter()
                .filter_map(|(player_actor, player_transform)| {
                    let position = Vec2::from(player_transform.translation);
                    //(possibliy based on other players in same lane - or start arena division based on highest y from players actor)
                    let mut has_ball = false;
                    if let Some(entity_with_ball) = ball_possession.get() {
                        has_ball = entity_with_ball == a.0
                    }
                    if position.y > y_min && position.y < y_max {
                        Some((player_actor.act_action, position, has_ball))
                    } else {
                        None
                    }
                })
                .collect();
            (a, player_data)
        })
        .collect();

    //currently only difference between having and not having possession, is how much would ai move in x axis
    for ((ai_entity, ai_position, ai_role, lane_num), player_data) in ai_actors_data {
        if ai_role.is_none() {
            let (y_min, y_max) = get_lane_bounds(lane_num, lane_height);

            let mut target_player_position = None;
            for (_player_action, player_position, _has_ball) in player_data.into_iter() {
                if target_player_position.is_none() || ai_position.distance(player_position) < ai_position.distance(target_player_position.unwrap()) {
                    target_player_position = Some(player_position);
                }
            }
            let target_player_position = target_player_position.unwrap_or(Vec2::new(
                if player_has_ball { ai_position.x - 100.0 } else { ai_position.x + 100.0 }, //TODO determine values based on AI goal post direction
                ai_position.y
            ));

            let delta_signum = (target_player_position - ai_position).x.signum();
            let (range_x_l, range_x_r) =  if player_has_ball {
                //when don't have a ball possession then move to position to try to mantain distance from actors in same lane
                //but, should have some system in place to assume that actors in lane would move forward, and to determine to meet them halfway, but stop sooner to be able to tackle them
                (target_player_position.x, target_player_position.x - (20.0*delta_signum))
            } else {
                //when do have ball possession, move forwards
                //(by not having any actor player in ai actor lane, or again based on player actor y, determine how much would be viable to move forward)
                (target_player_position.x - (20.0*delta_signum), target_player_position.x + (40.0*delta_signum))
            };
            let x_min = if range_x_l > range_x_r { range_x_r } else { range_x_l };
            let x_max = if range_x_l > range_x_r { range_x_l } else { range_x_r };
            let y = rng.gen_range(y_min..y_max);
            let x = rng.gen_range(x_min..x_max);

            let (_entity, mut ai_actor, _transform, mut ai) = query_actors.q1_mut().get_mut(ai_entity).unwrap();
            ai_actor.queue_action(actor::ActorAction::Running { x: x, y: y });
            ai.role = Some(AiRole::Move);
        }
    }

    // for (entity, actor, transform, _ai) in query_actors.q1_mut().iter_mut() {
    //     match actor.act_action {
    //         actor::ActorAction::Running { x, y } => {
    //             let he = helpers::spawn_movement_helper(
    //                 &mut commands,
    //                 &helper_materials,
    //                 Vec2::new(x, y),
    //                 Vec2::new(transform.translation.x, transform.translation.y),
    //                 entity.clone(),
    //                 helpers::HelperType::Run
    //             );
    //             commands.entity(he).insert(AiControlled::new());
    //         }
    //         _ => ()
    //     };
    // }

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