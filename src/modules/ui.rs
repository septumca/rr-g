use bevy::prelude::*;

use super:: {
    actor,
    team,
    states,
    utils,
    matchup,
};


pub struct SelectedText;
pub struct StateText;
pub struct ControlModeText;
pub struct GameText;
pub struct ScoreText;

pub const UI_SIZE: f32 = 20.0;

pub struct FontMaterials {
    debug_font: Handle<Font>
}

pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    clicked: Handle<ColorMaterial>,
    disabled: Handle<ColorMaterial>,
}

#[derive(Debug, PartialEq)]
pub enum ButtonStates {
    Normal,
    Hovered,
    Clicked,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonAction {
    Throw,
    Run,
    Play,
}

pub enum ButtonEvent {
    TriggerAction { source: Entity, action: ButtonAction, group: Option<u32> },
    ButtonGroupMemberClicked { source: Entity, group: u32 },
    DisableButtons { entities: Vec<Entity> },
}

pub struct ButtonGroup(pub u32);
pub struct RRButton {
    state: ButtonStates
}

const DEBUG_TEXT_SIZE: f32 = 10.0;
const DEBUG_OFF_SET_X: f32 = 20.0;
const DEBUG_OFF_SET_Y: f32 = 40.0;
const GAME_INFO_TEXT_SIZE: f32 = 32.0;
const CONTROL_BUTTON_GROUP: u32 = 1;

fn update_text(text: &mut Text, value: String) {
    text.sections[0].value = value;
}

pub fn setup_ui_materials(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>
) {
    commands.insert_resource(FontMaterials {
        debug_font: asset_server.load("FiraCode-Medium.ttf"),
    });
    commands.insert_resource(ButtonMaterials {
        normal: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
        hovered: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
        clicked: materials.add(Color::rgb(0.95, 0.95, 0.95).into()),
        disabled: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
    });
}

fn create_debug_text_bundle(fonts: &Res<FontMaterials>, text: String, y: f32) -> TextBundle {
    create_text_bundle(fonts, text, DEBUG_OFF_SET_X + 5.0, DEBUG_OFF_SET_Y + y, DEBUG_TEXT_SIZE, AlignSelf::FlexStart)
}

fn create_pre_game_text(fonts: &Res<FontMaterials>, text: String, y: f32) -> TextBundle {
    create_text_bundle(fonts, text.clone(), utils::WIN_W/2.0 - (text.len() as f32)*15.0/2.0, y, GAME_INFO_TEXT_SIZE, AlignSelf::FlexStart)
}

fn create_text_bundle(fonts: &Res<FontMaterials>, text: String, x: f32, y: f32, font_size: f32, alignment: AlignSelf) -> TextBundle {
    TextBundle {
        style: Style {
            align_self: alignment,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(y),
                left: Val::Px(x),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text {
            sections: vec![
                TextSection {
                    value: text,
                    style: TextStyle {
                        font: fonts.debug_font.clone(),
                        font_size: font_size,
                        color: Color::WHITE,
                    },
                }
            ],
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn spawn_debug_ui(mut commands: Commands, fonts: Res<FontMaterials>) {
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No entity".to_string(), 5.0))
        .insert(SelectedText);
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No state".to_string(), 17.0))
        .insert(StateText);
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No control mode".to_string(), 29.0))
        .insert(ControlModeText);
}

pub fn spawn_score_text(
    mut commands: Commands,
    fonts: Res<FontMaterials>,
    matchup: Res<matchup::Matchup>,
) {
    let text = format!("{}-{}", matchup.score_home, matchup.score_away);
    let bundle = create_text_bundle(&fonts, text.clone(), utils::WIN_W - (text.len() as f32)*10.0, 2.0, 16.0, AlignSelf::FlexStart);
    commands
        .spawn_bundle(bundle)
        .insert(ScoreText);
}

pub fn add_pre_game_text(
    mut commands: Commands,
    fonts: Res<FontMaterials>
) {
    let text = "Press Enter to start the game".to_owned();
    commands
        .spawn_bundle(create_pre_game_text(&fonts, text, 300.0))
        .insert(GameText);
}

pub fn add_score_text(
    mut commands: Commands,
    fonts: Res<FontMaterials>,
    matchup: Res<matchup::Matchup>,
) {
    let text_top = format!("{:?} team scores, score is now {} - {}", team::get_oposing_team(matchup.serving_side), matchup.score_home, matchup.score_away);
    let text_bottom = "Press Enter to continue".to_owned();
    commands
        .spawn_bundle(create_pre_game_text(&fonts, text_top, 300.0))
        .insert(GameText);
    commands
        .spawn_bundle(create_pre_game_text(&fonts, text_bottom, 350.0))
        .insert(GameText);
}

pub fn clear_game_text(
    mut commands: Commands,
    query: Query<Entity, With<GameText>>
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn control_mode_changed(
    control_mode: Res<actor::CurrentControlMode>,
    mut query_text: Query<&mut Text, With<ControlModeText>>
) {
    if control_mode.is_changed() {
        if let Ok(mut text) = query_text.single_mut() {
            update_text(&mut text, format!("Control mode: {:?}", control_mode.0));
        }
    }
}

fn state_changed(
    app_state: Res<State<states::AppState>>,
    mut query_text: Query<&mut Text, With<StateText>>
) {
    if app_state.is_changed() {
        if let Ok(mut text) = query_text.single_mut() {
            update_text(&mut text, format!("State: {:?}", app_state.current()));
        }
    }
}

fn selected_actor_changed(
    query_actor: Query<(Entity, &actor::Actor, &actor::BallPossession), With<actor::Selected>>,
    mut query_text: Query<&mut Text, With<SelectedText>>
) {
    let q_result = query_actor.single();
    let msg = if q_result.is_ok() {
        let (entity, actor, ball_possession) = q_result.unwrap();
        format!("Selected actor: {:?}, state: {:?}, has ball: {}", entity, actor.act_action, ball_possession.0)
    } else {
        format!("No actor selected")
    };
    if let Ok(mut text) = query_text.single_mut() {
        update_text(&mut text, msg);
    }
}

fn score_changed(
    matchup: Res<matchup::Matchup>,
    mut query_text: Query<&mut Text, With<ScoreText>>,
) {
    if matchup.is_changed() {
        if let Ok(mut text) = query_text.single_mut() {
            update_text(&mut text, format!("{}-{}", matchup.score_home, matchup.score_away));
        }
    }
}

pub fn ui_changes_listeners() -> SystemSet {
    SystemSet::new()
        .with_system(control_mode_changed.system())
        .with_system(state_changed.system())
        .with_system(selected_actor_changed.system())
        .with_system(score_changed.system())
}

pub fn create_button_bundles(
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    text: String,
    fonts: &Res<FontMaterials>,
    button_materials: &Res<ButtonMaterials>
) -> (ButtonBundle, TextBundle) {
    let b = ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(w), Val::Px(h)),
            margin: Rect::all(Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(y),
                left: Val::Px(x),
                ..Default::default()
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        material: button_materials.normal.clone(),
        ..Default::default()
    };

    let t = TextBundle {
        text: Text::with_section(
            text,
            TextStyle {
                font: fonts.debug_font.clone(),
                font_size: 16.0,
                color: Color::rgb(0.1, 0.1, 0.1),
            },
            Default::default(),
        ),
        ..Default::default()
    };

    (b, t)
}

pub fn create_button_entity(
    commands: &mut Commands,
    button_bundle: ButtonBundle,
    text_bundle: TextBundle,
    state: ButtonStates,
) -> Entity {
    commands
        .spawn_bundle(button_bundle)
        .with_children(|parent| {
            parent.spawn_bundle(text_bundle);
        })
        .insert(RRButton { state })
        .id()
}

pub fn spawn_buttons(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    fonts: Res<FontMaterials>,
) {
    let (button_throw, text_throw) = create_button_bundles(50.0, 20.0, 0.0, 0.0, "Throw".to_owned(), &fonts, &button_materials);
    let e_b_throw = create_button_entity(&mut commands, button_throw, text_throw, ButtonStates::Disabled);
    commands.entity(e_b_throw).insert(ButtonGroup(CONTROL_BUTTON_GROUP));
    commands.entity(e_b_throw).insert(ButtonAction::Throw);

    let (button_run, text_run) = create_button_bundles(50.0, 20.0, 55.0, 0.0, "Run".to_owned(), &fonts, &button_materials);
    let e_b_run = create_button_entity(&mut commands, button_run, text_run, ButtonStates::Disabled);
    commands.entity(e_b_run).insert(ButtonGroup(CONTROL_BUTTON_GROUP));
    commands.entity(e_b_run).insert(ButtonAction::Run);

    let (button_play, text_play) = create_button_bundles(40.0, 20.0, utils::WIN_W/2.0 - 20.0, 0.0, "Play".to_owned(), &fonts, &button_materials);
    let e_b_play = create_button_entity(&mut commands, button_play, text_play, ButtonStates::Disabled);
    commands.entity(e_b_play).insert(ButtonAction::Play);
}

pub fn button_state_changed(
    mut query: Query<(&RRButton, &mut Handle<ColorMaterial>), Changed<RRButton>>,
    button_materials: Res<ButtonMaterials>
) {
    for (button, mut material) in query.iter_mut() {
        match button.state {
            ButtonStates::Normal => {
                *material = button_materials.normal.clone();
            },
            ButtonStates::Clicked => {
                *material = button_materials.clicked.clone();
            },
            ButtonStates::Hovered => {
                *material = button_materials.hovered.clone();
            },
            ButtonStates::Disabled => {
                *material = button_materials.disabled.clone();
            },
        }
    }
}

pub fn button_interactions(
    mut interaction_query: Query<
        (Entity, &mut RRButton, &Interaction, Option<&ButtonGroup>, &ButtonAction),
        Changed<Interaction>,
    >,
    mut events: EventWriter<ButtonEvent>,
) {
    for (entity, mut button, interaction, button_group, button_action) in interaction_query.iter_mut() {
        if button.state == ButtonStates::Disabled {
            continue;
        }
        match *interaction {
            Interaction::Clicked => {
                button.state = ButtonStates::Clicked;
                events.send(ButtonEvent::TriggerAction {
                    source: entity,
                    action: *button_action,
                    group: button_group.and_then(|bg: &ButtonGroup| -> Option<u32> { Some(bg.0) }),
                });
            }
            Interaction::Hovered => {
                if button_group.is_none() || button.state != ButtonStates::Clicked {
                    button.state = ButtonStates::Hovered;
                }
            }
            Interaction::None => {
                if button_group.is_none() || button.state != ButtonStates::Clicked {
                    button.state = ButtonStates::Normal;
                }
            }
        }
    }
}

pub fn handle_button_events(
    mut events_r: EventReader<ButtonEvent>,
    mut query_buttons: Query<(Entity, &mut RRButton, &ButtonGroup)>,
    mut control_mode: ResMut<actor::CurrentControlMode>,
    mut app_state: ResMut<State<states::AppState>>,
) {
    for ev in events_r.iter() {
        match ev {
            ButtonEvent::TriggerAction { source, group, action } => {
                match *action {
                    ButtonAction::Run => {
                        control_mode.0 = actor::ControlMode::Run;
                    },
                    ButtonAction::Throw => {
                        control_mode.0 = actor::ControlMode::Throw;
                    },
                    ButtonAction::Play => {
                        app_state.set(states::AppState::Play).unwrap();
                    },
                };

                if let Some(bg) = group {
                    for (entity, mut button, button_group) in query_buttons.iter_mut() {
                        if button_group.0 == *bg && *source != entity {
                            button.state = ButtonStates::Normal;
                        }
                    }
                }
            },
            ButtonEvent::ButtonGroupMemberClicked { source, group } => {
                for (entity, mut button, button_group) in query_buttons.iter_mut() {
                    if button_group.0 == *group && *source != entity {
                        button.state = ButtonStates::Normal;
                    }
                }
                if let Ok(mut clicked_button) = query_buttons.get_component_mut::<RRButton>(*source) {
                    clicked_button.state = ButtonStates::Clicked;
                }
            },
            ButtonEvent::DisableButtons { entities } => {
                for e in entities.iter() {
                    if let Ok(mut button) = query_buttons.get_component_mut::<RRButton> (*e) {
                        button.state = ButtonStates::Disabled;
                    }
                }
            },
        };
    }
}

pub fn disable_buttons(
    mut query: Query<&mut RRButton>
) {
    for mut button in query.iter_mut() {
        button.state = ButtonStates::Disabled;
    }
}

pub fn enable_buttons(
    mut query: Query<&mut RRButton>
) {
    for mut button in query.iter_mut() {
        button.state = ButtonStates::Normal;
    }
}