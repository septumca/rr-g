use bevy::prelude::*;
use heron::prelude::*;

const PLAYER_SPEED: f32 = 100.0;
pub struct PlayerTextures(Handle<TextureAtlas>);

pub type AnimationTimer = Timer;

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

pub type TargetPosition = Vec3;
pub enum ActorState {
    Idle,
    Running,
    // Diving,
    Recovering
}
pub struct Actor {
    pub state: ActorState
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
        .insert(Actor { state: ActorState::Idle })
        .insert(Animation::new(vec![0]))
        .insert(AnimationTimer::from_seconds(1.0/8.0, true))
        .insert(Body::Capsule { half_segment: 4.0, radius: 10.0 })
        .insert(RotationConstraints::lock())
        // .insert(PhysicMaterial {
        //     restitution: 0.0, // Define the restitution. Higher value means more "bouncy"
        //     density: 80.0, // Define the density. Higher value means heavier.
        //     friction: 1.0, // Define the friction. Higher value means higher friction.
        // })
        .insert(Velocity::from(Vec2::ZERO));
}

pub fn handle_player_state(
    mut query: Query<(&Actor, &mut Animation), Changed<Actor>>,
) {
    for (actor, mut animation) in query.iter_mut() {
        animation.act_frame_index = 0;
        match actor.state {
            ActorState::Idle => {
                animation.sprite_indexes = vec![0];
            },
            ActorState::Running => {
                animation.sprite_indexes = vec![0, 1, 0, 2];
            },
            ActorState::Recovering => {
                animation.sprite_indexes = vec![4];
            }
        };
    }
}

pub fn stop_players(
    mut query: Query<&mut Velocity, With<Actor>>,
) {
    for mut velocity in query.iter_mut() {
        velocity.linear = Vec3::ZERO;
    }
}

pub fn trigger_move_players(
    mut query: Query<(&mut Actor, &TargetPosition, &Transform, &mut TextureAtlasSprite, &mut Velocity)>,
) {
    for (mut actor, target_position, transform, mut sprite, mut velocity) in query.iter_mut() {
        let delta = (*target_position - transform.translation).normalize() * PLAYER_SPEED;
        sprite.flip_x = delta.x < 0.0;
        actor.state = ActorState::Running;
        velocity.linear = Vec3::new(delta.x, delta.y, 0.0);
    }
}

pub fn player_reached_position(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Actor, &TargetPosition, &mut Transform, &mut Velocity)>,
    query_movement_helper: Query<(Entity, &super::helpers::MovementHelper)>
) {
    for (entity, mut actor, target_position, mut transform, mut velocity) in query.iter_mut() {
        let d_x = transform.translation.x - target_position.x;
        let d_y = transform.translation.y - target_position.y;
        if d_x.abs() < 1.0 && d_y.abs() < 1.0 {
            commands.entity(entity).remove::<TargetPosition> ();
            velocity.linear = Vec3::ZERO;
            transform.translation.x = target_position.x;
            transform.translation.y = target_position.y;
            actor.state = ActorState::Idle;

            for (helper_entity, player_entity) in query_movement_helper.iter() {
                if player_entity.player == entity {
                    commands.entity(helper_entity).despawn_recursive();
                }
            }
            return;
        }
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, &mut Animation)>,
) {
    for (mut timer, mut sprite, mut animation) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            animation.update();
            sprite.index = animation.get_sprite_index();
        }
    }
}

pub fn cleanup_movement(
    mut commands: Commands,
    mut query_players: Query<(Entity, &mut Actor), With<TargetPosition>>,
) {
    for (player, mut actor) in query_players.iter_mut() {
        commands.entity(player).remove::<super::player::TargetPosition> ();
        actor.state = ActorState::Idle;
    }
}

pub fn handle_collisions(
    mut events: EventReader<CollisionEvent>,
    mut commands: Commands,
    mut query: Query<(&mut Actor, &mut Velocity)>,
    query_movement_helper: Query<(Entity, &super::helpers::MovementHelper)>
) {
    for event in events.iter() {
        match event {
            CollisionEvent::Started(e1, e2) => {
                commands.entity(*e1).remove::<TargetPosition> ();
                commands.entity(*e2).remove::<TargetPosition> ();
                for (helper_entity, player_entity) in query_movement_helper.iter() {
                    if player_entity.player == *e1 || player_entity.player == *e2 {
                        commands.entity(helper_entity).despawn_recursive();
                    }
                }

                for (mut actor, mut velocity) in query.get_mut(*e1) {
                    actor.state = ActorState::Recovering;
                    velocity.linear = -velocity.linear*0.1;
                }

                for (mut actor, mut velocity) in query.get_mut(*e2) {
                    actor.state = ActorState::Recovering;
                    velocity.linear = -velocity.linear*0.1;
                }
            }
            _ => ()
            // CollisionEvent::Stopped(e1, e2) => {
            //     println!("Collision stopped between {:?} and {:?}", e1, e2)
            // }
        }
    }
}