use bevy::prelude::*;
use heron::prelude::*;
use std::time::Duration;

const PLAYER_SPEED: f32 = 100.0;
pub struct PlayerTextures(Handle<TextureAtlas>);

pub struct AnimationTimer(Timer);
pub struct ActionTimer(Timer);

pub struct Animation {
    pub act_frame_index: usize,
    pub sprite_indexes: Vec<usize>
}
impl Animation {
    pub fn new(sprite_indexes: Vec<usize>) -> Self {
        Self {
            act_frame_index: 0,
            sprite_indexes
        }
    }
    pub fn update(&mut self) {
        self.act_frame_index = (self.act_frame_index + 1) % self.sprite_indexes.len()
    }
    pub fn get_sprite_index(&self) -> u32 {
        return self.sprite_indexes[self.act_frame_index] as u32;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ActorAction {
    Idle,
    Running { x: f32, y: f32 },
    // Diving,
    Recovering(f32)
}
pub struct Actor {
    pub act_action: ActorAction,
    queued_action: Option<ActorAction>
}
impl Actor {
    pub fn new_idle() -> Self {
        Self {
            act_action: ActorAction::Idle,
            queued_action: None
        }
    }
    pub fn trigger_queued_action(&mut self) {
        self.act_action = self.queued_action.unwrap_or(ActorAction::Idle);
    }
    pub fn set_action(&mut self, action: ActorAction) {
        self.act_action = action;
        self.queued_action = None;
    }
    pub fn queue_action(&mut self, action: ActorAction) {
        match self.act_action {
            ActorAction::Idle => {
                self.set_action(action);
            },
            _ => {
                self.queued_action = Some(action);
            }
        }
    }
}




pub struct Selected {}


pub fn setup_player_sprites(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("players-red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 5, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.insert_resource(PlayerTextures(texture_atlas_handle));
}


pub fn spawn_player(commands: &mut Commands, player_sprites: &Res<PlayerTextures>, position: Vec3) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: player_sprites.0.clone(),
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(Actor::new_idle())
        .insert(Animation::new(vec![0]))
        .insert(AnimationTimer(Timer::from_seconds(1.0/8.0, true)))
        .insert(ActionTimer(Timer::from_seconds(1.0, false)))
        .insert(super::collision::ColliderType::Player)
        .insert(Body::Capsule { half_segment: 4.0, radius: 10.0 })
        .insert(RotationConstraints::lock())
        .insert(PhysicMaterial {
            restitution: 0.1, // Define the restitution. Higher value means more "bouncy"
            density: 80.0, // Define the density. Higher value means heavier.
            friction: 1.0, // Define the friction. Higher value means higher friction.
        })
        .insert(Velocity::from(Vec2::ZERO));
}

pub fn reset_move_actions(
    mut query: Query<&mut Actor>,
) {
    for mut actor in query.iter_mut() {
        match actor.act_action {
            ActorAction::Running { x: _, y: _ } => {
                actor.set_action(ActorAction::Idle);
            },
            _ => ()
        }
    }
}

pub fn reset_action_timer(timer: &mut ActionTimer, t: f32) {
    timer.0.set_duration(Duration::from_secs_f32(t));
    timer.0.set_repeating(false);
    timer.0.reset();
}


pub fn update_players_actions(
    time: Res<Time>,
    mut query: Query<(&mut Actor, &mut Transform, &mut ActionTimer)>,
) {
    for (mut actor, transform, mut timer) in query.iter_mut() {
        let is_action_finished = match actor.act_action {
            ActorAction::Idle => false,
            ActorAction::Running { x, y} => {
                let d_x = transform.translation.x - x;
                let d_y = transform.translation.y - y;

                d_x.abs() < 1.0 && d_y.abs() < 1.0
            },
            ActorAction::Recovering(_) => {
                timer.0.tick(time.delta());
                timer.0.finished()
            }
        };
        if is_action_finished {
            reset_action_timer(&mut timer, 1.0);
            actor.trigger_queued_action();
        }
    }
}

pub fn handle_player_action_change(
    mut query: Query<(&Actor, &Transform, &mut Velocity, &mut TextureAtlasSprite, &mut Animation, &mut ActionTimer), Changed<Actor>>,
) {
    // println!("handle_player_action_change triggered");
    for (actor, transform, mut velocity, mut sprite, mut animation, mut timer) in query.iter_mut() {
        match actor.act_action {
            ActorAction::Idle => {
                animation.sprite_indexes = vec![0];
                velocity.linear = Vec3::ZERO;
            },
            ActorAction::Running { x, y} => {
                let delta = (Vec3::new(x, y, transform.translation.z) - transform.translation).normalize() * PLAYER_SPEED;
                sprite.flip_x = delta.x < 0.0;
                velocity.linear = Vec3::new(delta.x, delta.y, 0.0);
                animation.sprite_indexes = vec![0, 1, 0, 2];
            },
            ActorAction::Recovering(t) => {
                animation.sprite_indexes = vec![4];
                reset_action_timer(&mut timer, t);
            }
        };
    }
}

pub fn update_helpers(
    mut commands: Commands,
    mut query: Query<(Entity, &Actor)>,
    query_movement_helper: Query<(Entity, &super::helpers::MovementHelper)>
) {
    for (entity, actor) in query.iter_mut() {
        let should_keep_helpers = match actor.act_action {
            ActorAction::Running { x: _, y: _ } => true,
            _ => false,
        };

        if should_keep_helpers {
            continue;
        }

        for (helper_entity, player_entity) in query_movement_helper.iter() {
            if player_entity.player == entity {
                commands.entity(helper_entity).despawn_recursive();
            }
        }
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, &mut Animation)>,
) {
    for (mut timer, mut sprite, mut animation) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            animation.update();
            sprite.index = animation.get_sprite_index();
        }
    }
}
