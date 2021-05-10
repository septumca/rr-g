use bevy::prelude::*;

pub struct SelectedHelper {}
pub struct TargetPosition {
    pub player: Entity
}


pub struct HelperMaterials {
    selected: Handle<ColorMaterial>,
    target: Handle<ColorMaterial>
}

pub fn setup_helper_materials(commands: &mut Commands, asset_server: &Res<AssetServer>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(HelperMaterials {
        selected: materials.add(asset_server.load("selectbox.png").into()),
        target: materials.add(asset_server.load("targetpos.png").into()),
    });
}

pub fn spawn_selected_helper(commands: &mut Commands, helper_materials: &Res<HelperMaterials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: helper_materials.selected.clone(),
            sprite: Sprite::new(Vec2::new(34.0, 34.0)),
            ..Default::default()
        })
        .insert(SelectedHelper {});
}

pub fn spawn_targetpos_helper(commands: &mut Commands, helper_materials: &Res<HelperMaterials>, position: Vec2, player: Entity) {
    commands
        .spawn_bundle(SpriteBundle {
            material: helper_materials.target.clone(),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            transform: Transform::from_translation(Vec3::new(position.x + 16.0, position.y + 16.0, 0.3)),
            ..Default::default()
        })
        .insert(TargetPosition { player });
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