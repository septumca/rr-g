use bevy::prelude::*;
use bevy_rapier2d::na::Vector2;

pub const WIN_W: f32 = 800.0;
pub const WIN_H: f32 = 600.0;
pub const SPRITE_SIZE: f32 = 32.0;
pub const SCALE: f32 = 1.0;
pub const TRUE_SPRITE_SIZE: f32 = SPRITE_SIZE * SCALE;
pub const ACTOR_SPRITE_SIZE_W_PADDING: f32 = 48.0;
pub const PLAYING_FIELD_Z: f32 = 2.0;

pub fn transform_pos_window_to_screen(window_pos: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    Vec2::new(window_pos.x - WIN_W / 2.0, window_pos.y - WIN_H / 2.0)
}

pub fn is_point_in_square(point: &bevy::prelude::Vec2, rect_origin: &bevy::prelude::Vec3, rect_half_size: f32) -> bool {
    point.x > (rect_origin.x - rect_half_size) &&
        point.x < (rect_origin.x + rect_half_size) &&
        point.y > (rect_origin.y - rect_half_size) &&
        point.y < (rect_origin.y + rect_half_size)
}


pub fn get_rotated_vector(r: f32) -> Vector2<f32> {
    Vector2::new(r.cos(), r.sin())
}
