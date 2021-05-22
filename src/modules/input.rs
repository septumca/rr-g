use bevy::prelude::*;
use super::{
    actor,
    helpers,
    states,
    utils
};

pub fn handle_keyboard_input_pre_round(
    keyboard_input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<states::AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        app_state.set(states::AppState::MovingToStartPosition).unwrap();
    }
}

pub fn handle_keyboard_input(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<states::AppState>>,
    mut control_mode: ResMut<actor::CurrentControlMode>,
    query: Query<&actor::BallPossession, With<actor::Selected>>
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        app_state.set(states::AppState::Play).unwrap();
        keyboard_input.reset(KeyCode::Space); //according to https://bevy-cheatbook.github.io/programming/states.html#with-input
    }
    if let Ok(ball_possession) = query.single() {
        if keyboard_input.just_pressed(KeyCode::Return) && ball_possession.0 {
            control_mode.0 = match control_mode.0 {
                actor::ControlMode::Throw => {
                    actor::ControlMode::Run
                }
                actor::ControlMode::Run => {
                    if query.single().is_ok() {
                        actor::ControlMode::Throw
                    } else {
                        actor::ControlMode::Run
                    }
                }
            };
        }
    }
}

pub fn handle_mouse_click(
    mut commands: Commands,
    mut query:  QuerySet<(
        Query<(Entity, &Transform), (With<actor::Actor>, Without<actor::Selected>)>,
        Query<(Entity, &Transform, &mut actor::Actor), With<actor::Selected>>,
    )>,
    query_movement_helper: Query<(Entity, &helpers::MovementHelper)>,
    mut control_mode: ResMut<actor::CurrentControlMode>,
    helper_materials: Res<helpers::HelperMaterials>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>
) {
    let mouse_left_pressed = mouse_input.just_pressed(MouseButton::Left);
    let mouse_right_pressed = mouse_input.just_pressed(MouseButton::Right);

    if mouse_right_pressed {
        for (prev_selected, _, _) in query.q1_mut().iter_mut() {
            commands.entity(prev_selected).remove::<actor::Selected> ();
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

    //get if some actor is clicked
    let mut clicked_entity = None;
    for (entity, transform) in query.q0().iter() {
        if utils::is_point_in_rect(&click_pos, &transform.translation, utils::TRUE_SPRITE_SIZE/2.0) {
            clicked_entity = Some(entity)
        }
    }

    //if it is, select him
    if clicked_entity.is_some() {
        for (prev_selected, _,  _) in query.q1_mut().iter_mut() {
            commands.entity(prev_selected).remove::<actor::Selected> ();
        }
        let clicked_entity = clicked_entity.unwrap();
        commands.entity(clicked_entity).insert(actor::Selected {});
        control_mode.0 = actor::ControlMode::Run;
        return;
    }

    //if not set target position
    for (selected, transform, mut actor) in query.q1_mut().iter_mut() {
        let action = match control_mode.0 {
            actor::ControlMode::Run => actor::ActorAction::Running { x: click_pos.x, y: click_pos.y },
            actor::ControlMode::Throw => actor::ActorAction::Throwing { x: click_pos.x, y: click_pos.y },
        };
        match actor.act_action {
            actor::ActorAction::Recovering(_) => {
                actor.queue_action(action);
            },
            _ => {
                actor.set_action(action);
            }
        }

        for (movement_helper, actor_entity) in query_movement_helper.iter() {
            if actor_entity.actor == selected {
                commands.entity(movement_helper).despawn_recursive();
            }
        }

        helpers::spawn_movement_helper(
            &mut commands,
            &helper_materials,
            Vec2::new(click_pos.x, click_pos.y),
            Vec2::new(transform.translation.x, transform.translation.y),
            selected.clone()
        );
    }
}