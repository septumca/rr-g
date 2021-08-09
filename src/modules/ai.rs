use core::f32;
use std::{cell::Cell, cmp::Ordering};

use bevy::prelude::*;
use bevy_rapier2d::{na::{Isometry, Isometry2, Point2, Vector2}, rapier::parry::{self, query::{Ray, RayCast}}};
use rand::prelude::*;

use crate::modules::utils::get_rotated_vector;

use super::{actor, arena, ball, helpers, round, team, utils};
pub struct PlayerControlled {}

const AI_FORWARD_MOMENTUM: f32 = 100.0;
const AI_WING_MARGIN: f32 = 100.0;
#[derive(Debug)]
pub struct AiControlled {
    pub role: Option<AiRole>,
    offense_focus: AiFocus,
    defense_focus: AiFocus,
}
impl AiControlled {
    pub fn get_focus(&self, intent: &AiTeamIntent) -> AiFocus {
        let mut rng = thread_rng();
        match intent {
            AiTeamIntent::Offense => self.offense_focus,
            AiTeamIntent::Defense => self.defense_focus,
            AiTeamIntent::Undecided => *vec![self.offense_focus, self.defense_focus].iter().choose(&mut rng).unwrap()
        }
    }
    pub fn assign(&mut self, role: AiRole) {
        self.role = Some(role);
    }
    pub fn reset(&mut self) {
        self.role = None;
    }
    pub fn new(offense_focus: AiFocus, defense_focus: AiFocus) -> Self {
        Self {
            role: None,
            offense_focus,
            defense_focus
        }
    }
    pub fn default() -> Self {
        Self {
            role: None,
            offense_focus: AiFocus::StayForward,
            defense_focus: AiFocus::DefendGoalPost,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AiRole {
    Move { x: f32, y: f32 },
    Guard,
}

//TODO: some better mechanism to distinguish between offense/defense?
#[derive(Debug, Clone, Copy)]
pub enum AiFocus {
    //always valid
    GetBall,
    Score,

    //offsense
    GuardBallCarrier,
    MoveOnWings,

    //defense
    StayForward,
    DefendGoalPost,
    InterceptBallCarrier
}
#[derive(Debug)]
pub enum AiTeamIntent {
    Offense,
    Defense,
    Undecided
}

#[derive(Debug, Clone, Copy)]
struct ActorWithBall {
    entity: Entity,
    is_ai: bool,
    position: Vec2,
    target_position: Option<Vec2>
}
struct AiActorData {
    entity: Entity,
    position: Vec2,
    role: Option<AiRole>,
    has_ball: bool,
}
struct PlayerActorData {
    entity: Entity,
    action: actor::ActorAction,
    position: Vec2,
    has_ball: bool
}


fn get_closest_from_query(
    ball_position: &Vec3,
    query: &Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>,
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

fn get_ai_team_intent(actor_with_ball: Option<ActorWithBall>) -> AiTeamIntent {
    if actor_with_ball.is_none() {
        return AiTeamIntent::Undecided;
    }
    let actor_with_ball = actor_with_ball.unwrap();
    if actor_with_ball.is_ai {
        AiTeamIntent::Offense
    } else {
        AiTeamIntent::Defense
    }
}

pub fn process_ai(
    mut commands: Commands,
    helper_materials: Res<helpers::HelperMaterials>,
    mut query_actors: QuerySet<(
        Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>,
        Query<(Entity, &mut actor::Actor, &Transform, &mut AiControlled)>,
        Query<(Entity, &actor::Actor, &Transform), With<PlayerControlled>>,
        Query<(&actor::Actor, &Transform, &AiControlled)>,
    )>,
    query_ball: Query<&Transform, With<ball::Ball>>,
    query_goal_posts: QuerySet<(
        Query<&Transform, (With<arena::GoalPost>, With<AiControlled>)>,
        Query<&Transform, (With<arena::GoalPost>, With<PlayerControlled>)>,
    )>,
    ball_possession: Res<ball::BallPossession>,
    arena: Res<arena::Arena>,
) {
    //TOOD data structures for actor carrying ball and ai team intent are not atomic - this needs to be looked at
    let mut rng = thread_rng();

    let mut actor_with_ball: Cell<Option<ActorWithBall>> = Cell::new(None);
    let ball_transform = query_ball.single();
    let player_goalpost_position = Vec2::from(query_goal_posts.q1().single().expect("Cannot get player_goalpost!").translation);
    let ai_goalpost_position = Vec2::from(query_goal_posts.q0().single().expect("Cannot get player_goalpost!").translation);

    if ball_possession.is_free() && ball_transform.is_ok() && actor_with_ball.get().is_none() {
        let ball_position = ball_transform.unwrap().translation;
        let (
            (closest_ai_entity, closest_ai_distance),
            (_closest_player_entity, closest_player_distance)
        ) = get_closest_from_query(&ball_position, query_actors.q0());
        let closest_ai_guard_distance = closest_ai_distance - (actor::PLAYER_GUARD_RADIUS - 10.0);

        let (_entity, mut _actor, transform, mut ai) = query_actors.q1_mut().get_mut(closest_ai_entity).unwrap();
        // println!("Distance AI and Player => {} < {}", closest_ai_distance, closest_player_distance);
        if closest_ai_distance < closest_player_distance {
            //WOULD TAKE THE BALL FIRST
            ai.assign(AiRole::Move { x: ball_position.x, y: ball_position.y });
            actor_with_ball.set(Some(ActorWithBall {
                entity: closest_ai_entity,
                is_ai: true,
                position: Vec2::from(transform.translation),
                target_position: Some(Vec2::new(ball_position.x, ball_position.y))
            }));
        } else if closest_ai_guard_distance < closest_player_distance {
            //WILL BE IN GUARD DISTANCE AT END OF THE ROUND
            let ai_position = Vec2::from(transform.translation);
            let ratio = closest_ai_guard_distance / closest_ai_distance;
            let position = ((Vec2::from(ball_position) - ai_position) * ratio) + ai_position;
            ai.assign(AiRole::Move { x: position.x, y: position.y });
        }
    }

    let mut ai_actors: Vec<AiActorData> = query_actors
        .q1_mut()
        .iter_mut()
        .map(|(entity, _actor, transform, ai)| -> AiActorData {
            let mut has_ball = false;
            if let Some(entity_with_ball) = ball_possession.get() {
                has_ball = entity_with_ball == entity;
            }
            if actor_with_ball.get().is_some() && has_ball {
                panic!("Actor {:?} has ball already - this shouldn't happen!", actor_with_ball.get().unwrap().entity);
            }
            let target_position = if let Some(ai_role) = ai.role {
                match ai_role {
                    AiRole::Move { x, y } => Some(Vec2::new(x, y)),
                    _ => None
                }
            } else {
                None
            };
            if has_ball {
                actor_with_ball.set(Some(ActorWithBall {
                    entity,
                    is_ai: true,
                    position: Vec2::from(transform.translation),
                    target_position
                }));
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
            if actor_with_ball.get().is_some() && has_ball {
                panic!("Actor {:?} has ball already - this shouldn't happen!", actor_with_ball.get().unwrap().entity);
            }
            if has_ball {
                actor_with_ball.set(Some(ActorWithBall {
                    entity,
                    is_ai: false,
                    position: Vec2::from(transform.translation),
                    target_position: None
                }));
            }
            PlayerActorData {
                entity,
                action: actor.act_action,
                position: Vec2::from(transform.translation),
                has_ball
            }
        })
        .collect();
    ai_actors.sort_by(|a, b| {
        if a.role.is_some() && b.role.is_none() {
            return Ordering::Greater;
        }
        if a.role.is_none() && b.role.is_some() {
            return Ordering::Less;
        }
        return Ordering::Equal;
    });
    let ai_team_intent = get_ai_team_intent(actor_with_ball.get());
    let default_target_position = player_goalpost_position;
    // println!("AI team intent is {:?} -> actor with ball is {:?}", ai_team_intent, actor_with_ball.get());
    //raytracing, start with straight line and gruadually deviate by some margin, find suitable vector
    //this works lot better, but need to somehow figure out how to steer actor to center of net
    //(i.e. dont start with 0 rotation but with rotation based on goal post center and then work around that)
    //instead of goal post center it can be also another target, e.g. some wing position or position near ball carrier


    for ai_actor_data in ai_actors.iter() {
        let signum = (player_goalpost_position.x - ai_actor_data.position.x).signum();
        if ai_actor_data.role.is_some() {
            continue;
        }

        //TODO now assign and target positions based on focus, which is based on team intent
        let ai = query_actors.q1_mut().get_component_mut::<AiControlled> (ai_actor_data.entity).expect("Cannot get AI actor!");
        let focus = if ai_actor_data.has_ball { AiFocus::Score } else { ai.get_focus(&ai_team_intent) };
        let target_position = match focus {
            //offsense
            AiFocus::MoveOnWings => {
                //TODO handle state when closing to players goalpost
                let distance_to_top = (arena.top - ai_actor_data.position.y).abs();
                let distance_to_bottom = (arena.bottom - ai_actor_data.position.y).abs();
                let (y_min, y_max) = if distance_to_top <= distance_to_bottom {
                    (arena.top-AI_WING_MARGIN, arena.top)
                } else {
                    (arena.bottom, arena.bottom+AI_WING_MARGIN)
                };
                Vec2::new(ai_actor_data.position.x + AI_FORWARD_MOMENTUM*signum, rng.gen_range(y_min..y_max))
            }
            AiFocus::GuardBallCarrier => {
                if let Some(bc) = actor_with_ball.get() {
                    if let Some(tp) = bc.target_position {
                        let signum_x = (ai_actor_data.position.x - player_goalpost_position.x ).signum();
                        let signum_y = (ai_actor_data.position.y - tp.y).signum();

                        let offset_x  = rng.gen_range(0.0..20.0) * signum_x;
                        let offset_y = rng.gen_range(30.0..60.0) * signum_y;

                        Vec2::new(tp.x + offset_x, tp.y + offset_y)
                    } else {
                        bc.position
                    }
                } else {
                    default_target_position
                }
            },
            AiFocus::Score => player_goalpost_position,
            //defense
            AiFocus::StayForward => default_target_position,
            AiFocus::DefendGoalPost => ai_goalpost_position,
            AiFocus::InterceptBallCarrier => default_target_position,
            _ => default_target_position
        };



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
            if let Ok(mut ai) = query_actors.q1_mut().get_component_mut::<AiControlled> (ai_actor_data.entity) {
                ai.role = Some(AiRole::Move { x: chm.x, y: chm.y });
                if let Some(bc) = actor_with_ball.get_mut() {
                    if bc.entity == ai_actor_data.entity {
                        bc.target_position = Some(Vec2::new(chm.x, chm.y));
                    }
                }
            }
        }
    }

    for (entity, mut actor, transform, ai) in query_actors.q1_mut().iter_mut() {
        if let Some(role) = ai.role {
            match role {
                AiRole::Move { x, y } => {
                    actor.set_action(actor::ActorAction::Running { x, y });
                },
                AiRole::Guard => ()
            };
        }

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
                commands.entity(he).insert(AiControlled::default());
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
        ai.reset();
    }
}