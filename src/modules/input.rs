use bevy::prelude::*;

use super::player;
use super::ui;
use super::utils;

pub fn handle_mouse_click(
    mut query: Query<(Entity, &Transform), (With<player::Actor>, Without<player::Selected>)>,
    mut query_selected: Query<(Entity, &mut player::Animation), (With<player::Actor>, With<player::Selected>)>,
    mut query_text: Query<&mut Text, With<ui::DiagText>>,
    mut commands: Commands,
    helper_materials: Res<super::helpers::HelperMaterials>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>
) {
    let mouse_left_pressed = mouse_input.just_pressed(MouseButton::Left);
    let mouse_right_pressed = mouse_input.just_pressed(MouseButton::Right);

    if mouse_right_pressed {
        for (prev_selected, _) in query_selected.iter_mut() {
            commands.entity(prev_selected).remove::<player::Selected> ();
            let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
            text.sections[0].value = format!("No entity selected");
        }
        return;
    }

    let click_pos = windows
        .get_primary()
        .and_then(|win| -> Option<bevy::prelude::Vec2> {
            if !mouse_left_pressed {
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
        if utils::is_point_in_rect(&click_pos, &transform.translation, 16.0) {
            clicked_entity = Some(entity)
        }
    }

    if clicked_entity.is_some() {
        for (prev_selected, _) in query_selected.iter_mut() {
            commands.entity(prev_selected).remove::<player::Selected> ();
        }
        let clicked_entity = clicked_entity.unwrap();
        let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
        text.sections[0].value = format!("Selected Entity: {:?}", clicked_entity);
        commands.entity(clicked_entity).insert(player::Selected {});
    } else {
        for (selected, mut animation) in query_selected.iter_mut() {
            commands.entity(selected).insert(player::TargetPosition::new(click_pos.x, click_pos.y, 1.0));
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0, 1, 0, 2];

            super::helpers::spawn_targetpos_helper(&mut commands, &helper_materials, Vec2::new(click_pos.x - 16.0, click_pos.y - 16.0), selected.clone());
        }
    }
}