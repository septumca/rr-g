use bevy::prelude::*;

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

pub struct Actor {}

pub enum ActorState {
    Idle,
    // Running,
    // Diving,
    // Recovering
}
pub struct Selected {}


pub fn setup_player_sprites(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("rr2.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 6, 1);
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
        .insert(Actor {})
        .insert(ActorState::Idle)
        .insert(Animation::new(vec![0]))
        .insert(AnimationTimer::from_seconds(1.0/8.0, true));
}

pub fn player_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &TargetPosition, &mut Transform, &mut Animation, &mut TextureAtlasSprite), With<Actor>>,
    query_tp_helper: Query<(Entity, &super::helpers::TargetPosition)>
) {
    for (entity, target_position, mut transform, mut animation, mut sprite) in query.iter_mut() {
        if transform.translation.x == target_position.x && transform.translation.y == target_position.y {
            commands.entity(entity).remove::<TargetPosition> ();
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0];

            for (helper_entity, player_entity) in query_tp_helper.iter() {
                if player_entity.player == entity {
                    commands.entity(helper_entity).despawn();
                }
            }
            return;
        }

        let delta = *target_position - transform.translation;
        sprite.flip_x = delta.x < 0.0;

        if delta.x.abs() < 1.0 && delta.y.abs() < 1.0 {
            transform.translation.x = target_position.x;
            transform.translation.y = target_position.y;
            return;
        }

        let delta = delta.normalize();
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