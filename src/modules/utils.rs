use bevy::prelude::*;

pub const WIN_W: f32 = 800.0;
pub const WIN_H: f32 = 600.0;

pub fn transform_pos_window_to_screen(window_pos: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    Vec2::new(window_pos.x - WIN_W / 2.0, window_pos.y - WIN_H / 2.0)
}

pub fn is_point_in_rect(point: &bevy::prelude::Vec2, rect_origin: &bevy::prelude::Vec3, rect_size: f32) -> bool {
    point.x > (rect_origin.x - rect_size) &&
        point.x < (rect_origin.x + rect_size) &&
        point.y > (rect_origin.y - rect_size) &&
        point.y < (rect_origin.y + rect_size)
}