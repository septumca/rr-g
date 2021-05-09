use bevy::prelude::*;

pub struct DiagText;

pub fn spawn_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font_path = "FiraCode-Medium.ttf";
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
                            font: asset_server.load(font_path.clone()),
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