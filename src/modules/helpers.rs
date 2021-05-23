use bevy::prelude::*;
use super::{actor, utils};


const LINE_THICKNESS: f32 = 2.0;

pub struct SelectedHelper {}
pub struct MovementHelper {
    pub actor: Entity
}

pub enum HelperType {
    Run,
    Throw,
}


pub struct HelperMaterials {
    pub selection: Handle<ColorMaterial>,
    pub movement_target: Handle<ColorMaterial>,
    pub movement_line: Handle<ColorMaterial>,
    pub throw_line: Handle<ColorMaterial>,
    pub tackle_zone: Handle<ColorMaterial>,
}

pub fn setup_helper_materials(commands: &mut Commands, asset_server: &Res<AssetServer>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(HelperMaterials {
        selection: materials.add(asset_server.load("selectbox.png").into()),
        movement_target: materials.add(asset_server.load("targetpos.png").into()),
        movement_line: materials.add(Color::rgb(0.67, 0.2, 0.2).into()),
        throw_line: materials.add(Color::rgb(0.8, 0.65, 0.1).into()),
        tackle_zone: materials.add(asset_server.load("tacklezone.png").into()),
    });
}

pub fn spawn_selected_helper(mut commands: Commands, helper_materials: Res<HelperMaterials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: helper_materials.selection.clone(),
            sprite: Sprite::new(Vec2::new(34.0, 34.0)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            ..Default::default()
        })
        .insert(SelectedHelper {});
}

pub fn deselect_all(
    mut commands: Commands,
    mut query_helper: Query<&mut Transform, With<SelectedHelper>>,
    query_selected: Query<Entity, (With<actor::Actor>, With<actor::Selected>)>
) {
    if let Ok(mut selected_helper_transform) = query_helper.single_mut() {
        selected_helper_transform.translation.z = -1.0;
    }
    for actor_selected in query_selected.iter() {
        commands.entity(actor_selected).remove::<actor::Selected> ();
    }
}

pub fn cleanup_movement_helpers(
    mut commands: Commands,
    query_helper: Query<Entity, With<MovementHelper>>,
) {
    for movement_helper in query_helper.iter() {
        commands.entity(movement_helper).despawn_recursive();
    }
}

pub fn spawn_movement_helper(
    commands: &mut Commands,
    helper_materials: &Res<HelperMaterials>,
    to: Vec2,
    from: Vec2,
    actor: Entity,
    htype: HelperType,
) {
    let material = match htype {
        HelperType::Run => helper_materials.movement_line.clone(),
        HelperType::Throw => helper_materials.throw_line.clone(),
    };
    let line_data = calculate_line(Vec2::ZERO, from - to);
    commands
        .spawn_bundle(SpriteBundle {
            material: helper_materials.movement_target.clone(),
            sprite: Sprite::new(Vec2::new(utils::SPRITE_SIZE, utils::SPRITE_SIZE)),
            transform: Transform::from_translation(Vec3::new(to.x, to.y, 0.3)),
            ..Default::default()
        })
        .insert(MovementHelper {
            actor
        })
        .with_children(|parent| {
            parent.
                spawn_bundle(SpriteBundle {
                    material,
                    sprite: Sprite::new(Vec2::new(line_data.0, LINE_THICKNESS)),
                    transform: Transform {
                        translation: line_data.1.0,
                        rotation: line_data.1.1,
                        scale: line_data.1.2
                    },
                    ..Default::default()
                });
        });
}

pub fn update_selected_helper(
    mut query: QuerySet<(
        Query<&mut Transform, With<SelectedHelper>>,
        Query<&Transform, (With<actor::Actor>, With<actor::Selected>)>
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

fn calculate_line(
    from: Vec2,
    to: Vec2
) -> (f32, (Vec3, Quat, Vec3)) {
    let midpoint = (from + to) / 2.0;
    let diff = from - to;
    let length = diff.length();
    let angle = Vec2::new(1.0, 0.0).angle_between(diff);

    (length, (Vec3::new(midpoint.x, midpoint.y, 0.3), Quat::from_rotation_z(angle), Vec3::splat(1.0)))
}
