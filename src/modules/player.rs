use bevy::prelude::*;


const PLAYER_SPEED: f32 = 2.0;
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
    // Recovering
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
    let texture_handle = asset_server.load("players-blue.png");
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
        .insert(AnimationTimer::from_seconds(1.0/8.0, true));
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
            }
        };
    }
}

pub fn player_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Actor, &TargetPosition, &mut Transform, &mut TextureAtlasSprite), With<Actor>>,
    query_movement_helper: Query<(Entity, &super::helpers::MovementHelper)>
) {
    for (entity, mut actor, target_position, mut transform, mut sprite) in query.iter_mut() {
        if transform.translation.x == target_position.x && transform.translation.y == target_position.y {
            commands.entity(entity).remove::<TargetPosition> ();
            actor.state = ActorState::Idle;

            for (helper_entity, player_entity) in query_movement_helper.iter() {
                if player_entity.player == entity {
                    commands.entity(helper_entity).despawn_recursive();
                }
            }
            return;
        }

        let delta = *target_position - transform.translation;
        sprite.flip_x = delta.x < 0.0;

        if delta.x.abs() < PLAYER_SPEED && delta.y.abs() < PLAYER_SPEED {
            transform.translation.x = target_position.x;
            transform.translation.y = target_position.y;
            return;
        }

        let delta = delta.normalize() * PLAYER_SPEED;
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
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
