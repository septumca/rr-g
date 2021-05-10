use bevy::prelude::*;

pub struct DiagText;

pub struct FontMaterials {
    debug_font: Handle<Font>
}

pub fn setup_ui_materials(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.insert_resource(FontMaterials {
        debug_font: asset_server.load("FiraCode-Medium.ttf"),
    });
}

pub fn spawn_ui(commands: &mut Commands, fonts: &Res<FontMaterials>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "No entity selected".to_string(),
                        style: TextStyle {
                            font: fonts.debug_font.clone(),
                            font_size: 10.0,
                            color: Color::WHITE,
                        },
                    }
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(DiagText);
}