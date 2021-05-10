use bevy::prelude::*;

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
    Running,
    Diving,
    Recovering
}
pub struct Selected {}


pub fn spawn_player(texture_atlas_handle: &Handle<TextureAtlas>, commands: &mut Commands, position: Vec3) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(Actor {})
        .insert(ActorState::Idle)
        .insert(Animation::new(vec![0]))
        .insert(AnimationTimer::from_seconds(1.0/8.0, true));
}

pub fn player_movement(mut query: Query<(Entity, &TargetPosition, &mut Transform, &mut Animation, &mut TextureAtlasSprite), With<Actor>>, mut commands: Commands) {
    for (entity, target_position, mut transform, mut animation, mut sprite) in query.iter_mut() {
        if transform.translation.x == target_position.x && transform.translation.y == target_position.y {
            commands.entity(entity).remove::<TargetPosition> ();
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0];
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