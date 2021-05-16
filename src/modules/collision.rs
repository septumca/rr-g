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

#[derive(PartialEq, Clone, Copy)]
pub enum ColliderType {
    Actor,
    Ball
}

fn resolve_actor_to_actor_start(
    mut actor: Mut<actor::Actor>,
) {
    actor.set_action(actor::ActorAction::Recovering(0.3));
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
                let e1 = Entity::from_bits(
                    collider_set
                        .get(idxl)
                        .unwrap()
                        .user_data as u64,
                );
                let e2 = Entity::from_bits(
                    collider_set
                        .get(idxr)
                        .unwrap()
                        .user_data as u64,
                );
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
    mut query: Query<&mut actor::Actor>
) {
    for event in events.iter() {
        let (e1, e1_type) = event.a;
        let (e2, e2_type) = event.b;

        if e1_type == ColliderType::Actor && e2_type == ColliderType::Actor {
            let actor = query.get_mut(e1).unwrap();
            resolve_actor_to_actor_start(actor);
            let actor = query.get_mut(e2).unwrap();
            resolve_actor_to_actor_start(actor);
        }
        else {
            let (ball_entity, actor_entity) = match e1_type {
                ColliderType::Ball => (e1, e2),
                ColliderType::Actor => (e2, e1)
            };
            let actor = query.get_mut(actor_entity).unwrap();
            let can_pickup_ball = match actor.act_action {
                actor::ActorAction::Recovering(_) | actor::ActorAction::Throwing { x: _, y: _ } => false,
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
