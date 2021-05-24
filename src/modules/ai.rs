use bevy::prelude::*;
use super::{
    actor,
    ball,
    helpers,
};
pub struct PlayerControlled {}
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

pub enum AiRole {
    GetBall,
    Move
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


pub fn process_ai(
    mut commands: Commands,
    helper_materials: Res<helpers::HelperMaterials>,
    mut query_actors: QuerySet<(
        Query<(Entity, &Transform, Option<&AiControlled>, Option<&PlayerControlled>), With<actor::Actor>>,
        Query<(&mut actor::Actor, &Transform, &mut AiControlled)>,
        Query<(Entity, &actor::Actor, &Transform), With<PlayerControlled>>,
    )>,
    query_ball: Query<&Transform, With<ball::Ball>>,
    ball_possession: Res<ball::BallPossession>
) {
    let ball_transform = query_ball.single();
    let mut ai_will_have_ball = false;
    if ball_possession.is_free() && ball_transform.is_ok() {
        let ball_position = ball_transform.unwrap().translation;

        let (
            (closest_ai_entity, closest_ai_distance),
            (_closest_player_entity, closest_player_distance)
        ) = get_closest_from_query(&ball_position, query_actors.q0());
        let closest_ai_guard_distance = closest_ai_distance - (actor::PLAYER_GUARD_RADIUS - 10.0);

        let (mut actor, transform, mut ai) = query_actors.q1_mut().get_mut(closest_ai_entity).unwrap();
        if closest_ai_distance < closest_player_distance || closest_ai_guard_distance > closest_player_distance {
            actor.queue_action(actor::ActorAction::Running { x: ball_position.x, y: ball_position.y });
            ai.role = Some(AiRole::GetBall);
            ai_will_have_ball = true;
        } else if closest_ai_guard_distance < closest_player_distance {
            let ai_position = Vec2::from(transform.translation);
            let ratio = closest_ai_guard_distance / closest_ai_distance;
            let position = ((Vec2::from(ball_position) - ai_position) * ratio) + ai_position;
            actor.queue_action(actor::ActorAction::Running { x: position.x, y: position.y });
            ai.role = Some(AiRole::Move);
        }

        match actor.act_action {
            actor::ActorAction::Running { x, y } => {
                helpers::spawn_movement_helper(
                    &mut commands,
                    &helper_materials,
                    Vec2::new(x, y),
                    Vec2::new(transform.translation.x, transform.translation.y),
                    closest_ai_entity.clone(),
                    helpers::HelperType::Run
                );
            }
            _ => ()
        };
        //incorporate result of this, i.e. if ai actor is close enough to grab a ball then expect that ai has possession in else(=other ai actors) branch
    }

    if let Some(entity_with_ball) = ball_possession.get() {
        let (_entity, _transform, ai_controlled, _player_controlled) = query_actors.q0().get(entity_with_ball).unwrap();
        ai_will_have_ball = ai_controlled.is_some();
    }

    //get number of ai actors that don't have assigned role
    //sort them by their y-axis position/(or role)
    //split areana into <number of unassigned ai actors> rows
    //assign row to each ai actor
    //move actor to/in this row
    if ai_will_have_ball {
        //when do have ball possession, move forwards
        //(by not having any actor player in ai actor lane, or again based on player actor y, determine how much would be viable to move forward)
    } else {
        //when don't have a ball possession then move backwards
        //(possibliy based on other players in same row - or start arena division based on highest y from players actor)
    }
    //what about throws? if we somehow determine that it would be benefical to throw then throw
    //(by comparing movements across ai actors, if there is possibility that some other ai actor would be able to move more forward, then pass the ball)
    //problem with throws is unlimited range (= implement range on throws, and/or moving idle actors to intercept ball)
    //or don't allow to score with throw (but this isn't probably good idea)
}