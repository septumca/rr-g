use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{EventQueue},
    rapier::{
        geometry::{ColliderSet, ContactEvent::Started},
    },
};
use super::{
    ball,
    actor,
};

pub struct RRCollisionEvent {
    a: (Entity, ColliderType),
    b: (Entity, ColliderType),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ColliderType {
    Actor,
    Ball,
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
                let collider_l = collider_set.get(idxl);
                let collider_r = collider_set.get(idxr);

                if collider_l.is_none() || collider_r.is_none() {
                    println!("Cannot get colliders from set!");
                    return;
                }

                let e1 = Entity::from_bits(collider_l.unwrap().user_data as u64);
                let e2 = Entity::from_bits(collider_r.unwrap().user_data as u64);
                let e1_type = query_type.get(e1).expect(format!("Entity {:?} doens't have ColliderType", e1).as_str());
                let e2_type = query_type.get(e2).expect(format!("Entity {:?} doens't have ColliderType", e1).as_str());

                ev_collision.send(RRCollisionEvent {
                    a: (e1, *e1_type),
                    b: (e2, *e2_type)
                });
            }
            _ => ()
        }
    }
}

pub fn handle_collision_events(
    mut events: EventReader<RRCollisionEvent>,
    mut events_ball: EventWriter<ball::BallEvent>,
    query: Query<&actor::Actor>,
    mut events_actor: EventWriter<actor::ActorEvents>,
) {
    for event in events.iter() {
        let (e1, e1_type) = event.a;
        let (e2, e2_type) = event.b;

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
        }
        else {
            let (ball_entity, actor_entity) = match e1_type {
                ColliderType::Ball => (e1, e2),
                ColliderType::Actor => (e2, e1),
            };
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
        }
    }
}
