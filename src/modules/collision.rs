use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{EventQueue},
    rapier::{
        geometry::{
            ColliderHandle,
            ColliderSet,
            ContactEvent::Started
        }
    }
};
use super::{
    actor,
    arena,
    ball
};

pub enum RRCollisionEventTypes {
    Contact,
    Intersection
}

pub struct RRCollisionEvent {
    // ev_type: RRCollisionEventTypes,
    a: (Entity, ColliderType),
    b: (Entity, ColliderType),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ColliderType {
    Actor,
    Ball,
    Wall,
    GoalPost,
}


pub fn get_entity_info_from_collider(
    collider_set: &Res<ColliderSet>,
    query_type: &Query<&ColliderType>,
    handle: ColliderHandle
) -> Option<(Entity, ColliderType)> {
    let collider = collider_set.get(handle);

    if collider.is_none() {
        println!("Cannot get colliders from set!");
        None
    } else {
        let e = Entity::from_bits(collider.unwrap().user_data as u64);
        let e_type = query_type.get(e).expect(format!("Entity {:?} doens't have ColliderType", e).as_str());

        Some((e, *e_type))
    }
}

pub fn send_rr_collision_event(
    collider_set: &Res<ColliderSet>,
    query_type: &Query<&ColliderType>,
    _ev_type: RRCollisionEventTypes,
    idxl: ColliderHandle,
    idxr: ColliderHandle,
    ev_collision: &mut EventWriter<RRCollisionEvent>,
) {
    let e1_data = get_entity_info_from_collider(&collider_set, &query_type, idxl);
    let e2_data = get_entity_info_from_collider(&collider_set, &query_type, idxr);

    if e1_data.is_some() && e2_data.is_some() {
        ev_collision.send(RRCollisionEvent {
            // ev_type,
            a: e1_data.unwrap(),
            b: e2_data.unwrap()
        });
    }
}

pub fn get_contact_events(
    events: Res<EventQueue>,
    collider_set: Res<ColliderSet>,
    query_type: Query<&ColliderType>,
    mut ev_collision: EventWriter<RRCollisionEvent>,
) {
    while let Ok(contact_event) = events.contact_events.pop() {
        match contact_event {
            Started(idxl, idxr) => {
                send_rr_collision_event(
                    &collider_set,
                    &query_type,
                    RRCollisionEventTypes::Contact,
                    idxl,
                    idxr,
                    &mut ev_collision
                );
            }
            _ => ()
        }
    }

    while let Ok(intersection_event) = events.intersection_events.pop() {
        send_rr_collision_event(
            &collider_set,
            &query_type,
            RRCollisionEventTypes::Intersection,
            intersection_event.collider1,
            intersection_event.collider2,
            &mut ev_collision
        );
    }
}

pub fn match_entity_pair_to_colliders(
    e1: Entity, e1_type: ColliderType,
    e2: Entity, e2_type:ColliderType,
    _collider_type1: ColliderType,
    _collider_type2: ColliderType
) -> Option<(Entity, Entity)> {
    let mut type1_result = None;
    let mut type2_result = None;
    if _collider_type1 == e1_type {
        type1_result = Some(e1);
    } else if _collider_type2 == e1_type {
        type2_result = Some(e1);
    }

    if _collider_type1 == e2_type {
        type1_result = Some(e2);
    } else if _collider_type2 == e2_type {
        type2_result = Some(e1);
    }

    if type1_result.is_some() && type2_result.is_some() {
        Some((type1_result.unwrap(), type2_result.unwrap()))
    } else {
        None
    }
}

pub fn handle_collision_events(
    mut events: EventReader<RRCollisionEvent>,
    mut events_ball: EventWriter<ball::BallEvent>,
    query: Query<&actor::Actor>,
    query_gp: Query<&arena::GoalPost>,
    mut events_actor: EventWriter<actor::ActorEvents>,
) {
    for event in events.iter() {
        let (e1, e1_type) = event.a;
        let (e2, e2_type) = event.b;

        println!("Collision between {:?} and {:?}", e1_type, e2_type);

        if e1_type == ColliderType::Actor && e2_type == ColliderType::Actor {
            let actor1 = query.get(e1).unwrap();
            let actor2 = query.get(e2).unwrap();

            events_actor.send_batch(
                vec![
                    actor::ActorEvents::ActorsCollided {
                        actor_entity: e1,
                        actor_action: actor1.act_action,
                        other_actor_entity: e2,
                        other_actor_action: actor2.act_action
                    },
                    actor::ActorEvents::ActorsCollided {
                        actor_entity: e2,
                        actor_action: actor2.act_action,
                        other_actor_entity: e1,
                        other_actor_action: actor1.act_action
                    }
                ].into_iter()
            );
            continue;
        }

        let collision_result = match_entity_pair_to_colliders(e1, e1_type, e2, e2_type, ColliderType::Ball, ColliderType::Actor);
        if let Some((ball_entity, actor_entity)) = collision_result {
            let actor = query.get(actor_entity).unwrap();
            let can_pickup_ball = match actor.act_action {
                actor::ActorAction::Recovering(_) | actor::ActorAction::Throwing { x: _, y: _ } | actor::ActorAction::Tackling { x: _, y: _ } => false,
                _ => true
            };
            if can_pickup_ball {
                events_ball.send(ball::BallEvent::Pickup {
                    actor_entity,
                    ball_entity
                });
            }
            continue;
        }

        //TODO: explicitly create handler for intersection events and handle it there?
        let collision_result = match_entity_pair_to_colliders(e1, e1_type, e2, e2_type, ColliderType::Ball, ColliderType::GoalPost);
        if let Some((_ball_entity, gp_entity)) = collision_result {
            let oposing_team = query_gp.get(gp_entity).unwrap().get_oposing_team();
            println!("{:?} team scores!", oposing_team);
            continue;
        }

        let collision_result = match_entity_pair_to_colliders(e1, e1_type, e2, e2_type, ColliderType::Ball, ColliderType::Wall);
        if let Some((ball_entity, _wall_entity)) = collision_result {
            events_ball.send(ball::BallEvent::WallBounce { ball_entity });
            continue;
        }
    }
}
