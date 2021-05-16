use bevy::prelude::*;

use super:: {
    player,
    states,
};


pub struct SelectedText;
pub struct StateText;
pub struct ControlModeText;

pub struct FontMaterials {
    debug_font: Handle<Font>
}

const TEXT_SIZE: f32 = 10.0;

fn update_text(text: &mut Text, value: String) {
    text.sections[0].value = value;
}

pub fn setup_ui_materials(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.insert_resource(FontMaterials {
        debug_font: asset_server.load("FiraCode-Medium.ttf"),
    });
}

fn create_debug_text_bundle(fonts: &Res<FontMaterials>, text: String, y: f32) -> TextBundle {
    TextBundle {
        style: Style {
            align_self: AlignSelf::FlexStart,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(y),
                left: Val::Px(5.0),
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
                        font_size: TEXT_SIZE,
                        color: Color::WHITE,
                    },
                }
            ],
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn spawn_ui(commands: &mut Commands, fonts: &Res<FontMaterials>) {
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No entity".to_string(), 5.0))
        .insert(SelectedText);
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No state".to_string(), 20.0))
        .insert(StateText);
    commands
        .spawn_bundle(create_debug_text_bundle(&fonts, "No control mode".to_string(), 35.0))
        .insert(ControlModeText);
}

fn control_mode_changed(
    control_mode: Res<player::CurrentControlMode>,
    mut query_text: Query<&mut Text, With<ControlModeText>>
) {
    if control_mode.is_changed() {
        let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
        update_text(&mut text, format!("Control mode: {:?}", control_mode.0));
    }
}

fn state_changed(
    app_state: Res<State<states::AppState>>,
    mut query_text: Query<&mut Text, With<StateText>>
) {
    if app_state.is_changed() {
        let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
        update_text(&mut text, format!("State: {:?}", app_state.current()));
    }
}

fn selected_player_changed(
    query_player: Query<(Entity, &player::Actor, &player::BallPossession), With<player::Selected>>,
    mut query_text: Query<&mut Text, With<SelectedText>>
) {
    let q_result = query_player.single();
    let msg = if q_result.is_ok() {
        let (entity, actor, ball_possession) = q_result.unwrap();
        format!("Selected player: {:?}, state: {:?}, has ball: {}", entity, actor.act_action, ball_possession.0)
    } else {
        format!("No player selected")
    };
    let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
    update_text(&mut text, msg);
}

pub fn ui_changes_listeners() -> SystemSet {
    SystemSet::new()
        .with_system(control_mode_changed.system())
        .with_system(state_changed.system())
        .with_system(selected_player_changed.system())
}