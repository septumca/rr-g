use bevy::prelude::*;

use super::player;
use super::ui;
use super::utils;

pub fn handle_keyboard_input(mut keyboard_input: ResMut<Input<KeyCode>>, mut app_state: ResMut<State<super::states::AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        app_state.set(super::states::AppState::Play).unwrap();
        keyboard_input.reset(KeyCode::Space); //according to https://bevy-cheatbook.github.io/programming/states.html#with-input
    }
}

pub fn handle_mouse_click(
    mut query:  QuerySet<(
        Query<(Entity, &Transform), (With<player::Actor>, Without<player::Selected>)>,
        Query<(Entity, &Transform, &mut player::Actor), With<player::Selected>>,
    )>,
    query_movement_helper: Query<(Entity, &super::helpers::MovementHelper)>,
    mut query_text: Query<&mut Text, With<ui::SelectedText>>,
    mut commands: Commands,
    helper_materials: Res<super::helpers::HelperMaterials>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>
) {
    let mouse_left_pressed = mouse_input.just_pressed(MouseButton::Left);
    let mouse_right_pressed = mouse_input.just_pressed(MouseButton::Right);

    if mouse_right_pressed {
        for (prev_selected, _, _) in query.q1_mut().iter_mut() {
            commands.entity(prev_selected).remove::<player::Selected> ();
            let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
            super::ui::update_text(&mut text, format!("No entity selected"));
        }
        return;
    }

    //get mouse position if mouse left button is clicked
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

    //get if some player is clicked
    let mut clicked_entity = None;
    for (entity, transform) in query.q0().iter() {
        if utils::is_point_in_rect(&click_pos, &transform.translation, 16.0) {
            clicked_entity = Some(entity)
        }
    }

    //if it is, select him
    if clicked_entity.is_some() {
        for (prev_selected, _,  _) in query.q1_mut().iter_mut() {
            commands.entity(prev_selected).remove::<player::Selected> ();
        }
        let clicked_entity = clicked_entity.unwrap();
        let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
        super::ui::update_text(&mut text, format!("Selected Entity: {:?}", clicked_entity));
        commands.entity(clicked_entity).insert(player::Selected {});
        return;
    }

    //if not set target position
    for (selected, transform, mut actor) in query.q1_mut().iter_mut() {
        let run_action = player::ActorAction::Running { x: click_pos.x, y: click_pos.y };
        match actor.act_action {
            player::ActorAction::Running { x: _, y: _ } => {
                actor.set_action(run_action);
            },
            _ => {
                actor.queue_action(run_action);
            }
        }

        for (movement_helper, player_entity) in query_movement_helper.iter() {
            if player_entity.player == selected {
                commands.entity(movement_helper).despawn_recursive();
            }
        }

        super::helpers::spawn_movement_helper(
            &mut commands,
            &helper_materials,
            Vec2::new(click_pos.x, click_pos.y),
            Vec2::new(transform.translation.x, transform.translation.y),
            selected.clone()
        );
    }
}