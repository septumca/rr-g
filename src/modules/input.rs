use bevy::prelude::*;

use super::player;
use super::ui;
use super::utils;

pub fn handle_mouse_click(
    mut query: Query<(Entity, &Transform), (With<player::Actor>, Without<player::Selected>)>,
    mut query_selected: Query<(Entity, &mut player::Animation), (With<player::Actor>, With<player::Selected>)>,
    mut query_text: Query<&mut Text, With<ui::DiagText>>,
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>
) {
    let click_pos = windows
        .get_primary()
        .and_then(|win| -> Option<bevy::prelude::Vec2> {
            if !mouse_input.just_pressed(MouseButton::Left) {
                return None;
            }
            win.cursor_position()
        })
        .and_then(|pos| -> Option<bevy::prelude::Vec2> {
            Some(utils::transform_pos_window_to_screen(pos))
        });
    if click_pos.is_none() {
        return;
    }
    let click_pos = click_pos.unwrap();

    let mut clicked_entity = None;
    for (entity, transform) in query.iter_mut() {
        if utils::is_actor_clicked(&transform.translation, click_pos) {
            clicked_entity = Some(entity)
        }
    }

    if clicked_entity.is_some() {
        for (prev_selected, _) in query_selected.iter_mut() {
            commands.entity(prev_selected).remove::<player::Selected> ();
        }
        let clicked_entity = clicked_entity.unwrap();
        for mut text in query_text.iter_mut() {
            text.sections[0].value = format!("Selected Entity: {:?}", clicked_entity.clone());
        }
        commands.entity(clicked_entity).insert(player::Selected {});
    } else {
        for (selected, mut animation) in query_selected.iter_mut() {
            commands.entity(selected).insert(player::TargetPosition::new(click_pos.x, click_pos.y, 1.0));
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0, 1, 0, 2];
        }
    }
}