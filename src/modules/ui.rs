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

pub struct FontMaterials {
    debug_font: Handle<Font>
}

const DEBUG_TEXT_SIZE: f32 = 10.0;
const DEBUG_OFF_SET_X: f32 = 20.0;
const DEBUG_OFF_SET_Y: f32 = 40.0;
const GAME_INFO_TEXT_SIZE: f32 = 32.0;

fn update_text(text: &mut Text, value: String) {
    text.sections[0].value = value;
}

pub fn setup_ui_materials(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.insert_resource(FontMaterials {
        debug_font: asset_server.load("FiraCode-Medium.ttf"),
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