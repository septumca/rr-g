use bevy::prelude::*;
use super::{actor, ai, ball, helpers, states, ui, utils};

pub fn handle_keyboard_input_pre_round(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<states::AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        app_state.set(states::AppState::MovingToStartPosition).unwrap();
        keyboard_input.reset(KeyCode::Return); //according to https://bevy-cheatbook.github.io/programming/states.html#with-input
    }
}

pub fn handle_keyboard_input(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<states::AppState>>,
    mut control_mode: ResMut<actor::CurrentControlMode>,
    ball_possession: Res<ball::BallPossession>,
    query: Query<Entity, With<actor::Selected>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        app_state.set(states::AppState::Play).unwrap();
        keyboard_input.reset(KeyCode::Space); //according to https://bevy-cheatbook.github.io/programming/states.html#with-input
    }
    if let Ok(entity) = query.single() {
        let has_ball = ball_possession.has_actor_ball(entity);
        if keyboard_input.just_pressed(KeyCode::Return) && has_ball {
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
        Query<(Entity, &Transform), (With<actor::Actor>, With<ai::PlayerControlled>, Without<actor::Selected>)>,
        Query<(Entity, &Transform, &mut actor::Actor), (With<actor::Selected>, With<ai::PlayerControlled>)>,
    )>,
    query_movement_helper: Query<(Entity, &helpers::MovementHelper)>,
    mut control_mode: ResMut<actor::CurrentControlMode>,
    helper_materials: Res<helpers::HelperMaterials>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    query_buttons: Query<(Entity, &ui::ButtonAction, &ui::ButtonGroup), With<ui::RRButton>>,
    mut event_buttons: EventWriter<ui::ButtonEvent>,
    ball_possession: Res<ball::BallPossession>,
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

    //if click is inside UI then return
    if click_pos.y >= utils::WIN_H/2.0 - ui::UI_SIZE {
        return;
    }

    //get if some actor is clicked
    let mut clicked_entity = None;
    let mut has_ball = false;
    for (entity, transform) in query.q0().iter() {
        if utils::is_point_in_square(&click_pos, &transform.translation, utils::TRUE_SPRITE_SIZE/2.0) {
            clicked_entity = Some(entity);
            has_ball = ball_possession.has_actor_ball(entity);
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

        if has_ball {
            for (entity, button_action, button_group) in query_buttons.iter() {
                match *button_action {
                    ui::ButtonAction::Run => {
                        event_buttons.send(ui::ButtonEvent::ButtonGroupMemberClicked {
                            source: entity,
                            group: button_group.0
                        });
                    },
                    _ => ()
                };
            }
        } else {
            let entities: Vec<Entity> = query_buttons.iter().map(|(entity, _button_action, _button_group)| { entity.clone() }).collect();
            event_buttons.send(ui::ButtonEvent::DisableButtons { entities });
        }

        return;
    }

    //if not set target position
    for (selected, transform, mut actor) in query.q1_mut().iter_mut() {
        let (action, htype) = match control_mode.0 {
            actor::ControlMode::Run => (actor::ActorAction::Running { x: click_pos.x, y: click_pos.y }, helpers::HelperType::Run),
            actor::ControlMode::Throw => (actor::ActorAction::Throwing { x: click_pos.x, y: click_pos.y }, helpers::HelperType::Throw),
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
            selected.clone(),
            htype
        );
    }
}