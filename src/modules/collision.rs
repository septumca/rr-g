use bevy::prelude::*;
use heron::prelude::*;

#[derive(PartialEq)]
pub enum ColliderType {
    Player,
    Ball
}

pub fn handle_collisions(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut query: Query<&mut super::player::Actor>,
    query_type: Query<&ColliderType>,
) {
    for event in events.iter() {
        match event {
            CollisionEvent::Started(e1, e2) => {
                //TODO: this could be handled better?
                let e1_type = query_type.get(*e1).expect(format!("Entity {:?} doens't have ColliderType", e1).as_str());
                let e2_type = query_type.get(*e2).expect(format!("Entity {:?} doens't have ColliderType", e1).as_str());

                if *e1_type == ColliderType::Player && *e2_type == ColliderType::Player {
                    for mut actor in query.get_mut(*e1) {
                        actor.set_action(super::player::ActorAction::Recovering(0.3));
                    }

                    for mut actor in query.get_mut(*e2) {
                        actor.set_action(super::player::ActorAction::Recovering(0.3));
                    }
                }
                else {
                    match e1_type {
                        ColliderType::Ball => {
                            println!("ball picked up!");
                            commands.entity(*e1).despawn();
                        },
                        ColliderType::Player => {
                            println!("player got ball");
                            //TODO: pickup ball
                        }
                    }

                    match e2_type {
                        ColliderType::Ball => {
                            println!("ball picked up!");
                            commands.entity(*e2).despawn();
                        },
                        ColliderType::Player => {
                            println!("player got ball");
                            //TODO: pickup ball
                        }
                    }
                }
            }
            _ => ()
            // CollisionEvent::Stopped(e1, e2) => {
            //     println!("Collision stopped between {:?} and {:?}", e1, e2)
            // }
        }
    }
}
