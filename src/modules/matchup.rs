use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodySet},
    }
};
use super::{actor, states, physics, team};

pub struct Matchup {
    pub score_home: u8,
    pub score_away: u8,
    actors: Vec<(Entity, Vec2, team::Team)>,
    pub serving_side: team::Team,
    pub ball_home_position: Vec2,
    pub ball_away_position: Vec2
}

pub enum MatchupEvents {
    Scored(team::Team, u8)
}

impl  Matchup {
    pub fn new(ball_home_position: Vec2, ball_away_position: Vec2) -> Self {
        Self {
            score_away: 0,
            score_home: 0,
            actors: vec![],
            serving_side: team::Team::Home,
            ball_home_position,
            ball_away_position,
        }
    }

    pub fn add_actors(&mut self, actors: Vec<(Entity, Vec2, team::Team)>) {
        self.actors.extend(actors);
    }

    pub fn add_score(&mut self, team: team::Team, amount: u8) {
        match  team {
            team::Team::Home => {
                self.score_home += amount;
            },
            team::Team::Away => {
                self.score_away += amount;
            }
        };
    }
}

pub fn move_actors_to_positions(
    mut query_actors: Query<(&mut actor::Actor, &RigidBodyHandleComponent)>,
    matchup: Res<Matchup>,
    mut rigid_body_set: ResMut<RigidBodySet>,
) {
    for (actor_entity, position, _team) in matchup.actors.iter() {
        if let Ok((mut actor, rigid_body_handle)) = query_actors.get_mut(*actor_entity) {
            // TODO: need to resolve collision between players and reset action
            // actor.set_action(actor::ActorAction::Running { x: position.x, y: position.y });
            // actor.queue_action(actor::ActorAction::Idle);
            physics::set_rb_properties(rigid_body_handle, &mut rigid_body_set, None, Some(*position), None);
            actor.set_action(actor::ActorAction::Running { x: position.x, y: position.y });
        }
    }
}

pub fn are_actors_in_position(
    query_actors: Query<&actor::Actor>,
    matchup: Res<Matchup>,
    mut app_state: ResMut<State<states::AppState>>,
) {
     let moving_actor = matchup.actors.iter().find(|(actor_entity, _position, _team)| -> bool {
        if let Ok(actor) = query_actors.get(*actor_entity) {
            match actor.act_action {
                actor::ActorAction::Lookout | actor::ActorAction::Idle => false,
                _ => true
            }
        } else {
            true
        }
    });

    if moving_actor.is_none() {
        app_state.set(states::AppState::Plan).unwrap();
    }
}

pub fn update_actors_facing(
    mut query_actors: Query<(&team::Team, &mut TextureAtlasSprite), With<actor::Actor>>
) {
    for (team, mut sprite) in query_actors.iter_mut() {
        match *team {
            team::Team::Away => {
                sprite.flip_x = true;
            },
            team::Team::Home => {
                sprite.flip_x = false;
            }
        }
    }
}

pub fn handle_matchup_events(
    mut events: EventReader<MatchupEvents>,
    mut matchup: ResMut<Matchup>,
    mut app_state: ResMut<State<states::AppState>>,
) {
    for event in events.iter() {
        match *event {
            MatchupEvents::Scored(team_scored_against, amount) => {
                matchup.add_score(team::get_oposing_team(team_scored_against), amount);
                app_state.set(states::AppState::Scored).unwrap();
                matchup.serving_side = team_scored_against;
            }
        }
    }
}