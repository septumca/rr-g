use bevy::prelude::*;
use std::cmp;

pub struct AnimationTimer(pub Timer);
pub struct Animation {
    act_frame_index: usize,
    sprite_indexes: Vec<usize>,
    pub looping: bool,
    pub finished: bool,
}
impl Animation {
    pub fn new(sprite_indexes: Vec<usize>) -> Self {
        Self {
            act_frame_index: 0,
            sprite_indexes,
            looping: true,
            finished: false
        }
    }
    pub fn update_sprites_indexes(&mut self, sprite_indexes: Vec<usize>, looping: bool) {
        self.act_frame_index = 0;
        self.sprite_indexes = sprite_indexes;
        self.looping = looping;
        self.finished = false;
    }
    pub fn update(&mut self) {
        if self.finished {
            return
        }
        self.act_frame_index = if self.looping {
            (self.act_frame_index + 1) % self.sprite_indexes.len()
        } else {
            let i = cmp::min(self.act_frame_index + 1, self.sprite_indexes.len() - 1);
            self.finished = i == self.sprite_indexes.len() - 1;
            i
        }
    }
    pub fn get_sprite_index(&self) -> u32 {
        return self.sprite_indexes[self.act_frame_index] as u32;
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