use bevy::prelude::*;

pub struct SelectedHelper {}

pub fn spawn_helpers(commands: &mut Commands, asset_server: &Res<AssetServer>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    let selected_texture_handle = asset_server.load("selectbox.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(selected_texture_handle.into()),
            sprite: Sprite::new(Vec2::new(34.0, 34.0)),
            ..Default::default()
        })
        .insert(SelectedHelper {});
}

pub fn update_selected_helper(
    mut query: QuerySet<(
        Query<&mut Transform, With<SelectedHelper>>,
        Query<&Transform, (With<super::player::Actor>, With<super::player::Selected>)>
    )>
) {
    let selected = query.q1().single();
    let pos = selected.ok().and_then(|s| -> Option<(f32, f32)> {
        Some((s.translation.x, s.translation.y))
    });
    let mut helper = query.q0_mut().single_mut().expect("Cannot get selected helper");

    if pos.is_none() {
        helper.translation.z = -1.0;
        return;
    }
    let pos = pos.unwrap();
    helper.translation.x = pos.0 - 0.5;
    helper.translation.y = pos.1 - 1.0;
    helper.translation.z = 0.5;
}