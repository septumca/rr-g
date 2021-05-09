use bevy::prelude::*;

pub const WIN_W: f32 = 800.0;
pub const WIN_H: f32 = 600.0;

pub fn transform_pos_window_to_screen(window_pos: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    Vec2::new(window_pos.x - WIN_W / 2.0, window_pos.y - WIN_H / 2.0)
}

pub fn is_actor_clicked(player_pos: &bevy::prelude::Vec3, click_pos: bevy::prelude::Vec2) -> bool {
    click_pos.x > (player_pos.x - 16.0) &&
        click_pos.x < (player_pos.x + 16.0) &&
        click_pos.y > (player_pos.y - 16.0) &&
        click_pos.y < (player_pos.y + 16.0)
}