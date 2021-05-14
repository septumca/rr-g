use bevy::prelude::*;


pub struct SelectedText;
pub struct StateText;
pub struct ActorStateText;

pub struct FontMaterials {
    debug_font: Handle<Font>
}

const TEXT_SIZE: f32 = 10.0;

pub fn update_text(text: &mut Text, value: String) {
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
        .spawn_bundle(create_debug_text_bundle(&fonts, "TODO: Actor Debug".to_string(), 35.0))
        .insert(ActorStateText);
}