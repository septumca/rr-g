use bevy::prelude::*;

use super::{
    collision,
    physics,
    team,
    utils,
};


pub struct Arena {
    pub width: f32,
    pub height: f32,
}
pub struct ArenaWall {}
pub struct GoalPost {
    pub team: team::Team
}

pub struct ArenaMaterials {
    pub wall: Handle<ColorMaterial>,
    pub ground: Handle<ColorMaterial>,
    pub blue_goal_post: Handle<ColorMaterial>,
    pub red_goal_post: Handle<ColorMaterial>,
}

pub fn setup_arena_materials(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>
) {
    commands.insert_resource(ArenaMaterials {
        wall: materials.add(Color::rgb(0.65, 0.65, 0.65).into()),
        ground: materials.add(Color::rgb(0.28, 0.44, 0.28).into()),
        blue_goal_post: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        red_goal_post: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
    });
}

pub fn spawn_wall(
    commands: &mut Commands,
    arena_materials: &Res<ArenaMaterials>,
    x: f32, y: f32, w: f32, h: f32
) {
    let wall_entity = commands
        .spawn_bundle(SpriteBundle {
            material: arena_materials.wall.clone(),
            sprite: Sprite::new(Vec2::new(w, h)),
            ..Default::default()
        })
        .insert(ArenaWall {})
        .insert(collision::ColliderType::Wall)
        .id();

    physics::create_physics_wall(commands, wall_entity, Vec2::new(x + w/2.0, y - h/2.0), w, h);
}

pub fn spawn_goal_post(
    commands: &mut Commands,
    arena_materials: &Res<ArenaMaterials>,
    team: team::Team,
    x: f32, y: f32, w: f32, h: f32
) {
    let material = match team {
        team::Team::Home => arena_materials.blue_goal_post.clone(),
        team::Team::Away => arena_materials.red_goal_post.clone(),
    };
    let gp_entity = commands
    .spawn_bundle(SpriteBundle {
        material,
        sprite: Sprite::new(Vec2::new(w, h)),
        ..Default::default()
    })
    .insert(GoalPost { team })
    .insert(collision::ColliderType::GoalPost)
    .id();

physics::create_physics_goalpost(commands, gp_entity, Vec2::new(x + w/2.0, y - h/2.0), w, h);
}

pub fn create_simple(
    commands: &mut Commands,
    arena_materials: &Res<ArenaMaterials>,
    w: f32,
    h: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let wall_thickness = 20.0;
    let goal_post_size = 100.0;

    let left = -(w+offset_x)/2.0 + offset_x;
    let top = (h+offset_y)/2.0 - offset_y;
    let right = left + w - wall_thickness;
    let bottom = top - h + wall_thickness;
    let vertical_section_size = (utils::WIN_H - offset_y - 2.0*wall_thickness - goal_post_size) / 2.0;

    commands.insert_resource(Arena { width: w, height: h });

    spawn_wall(commands, arena_materials, left, top, utils::WIN_W, wall_thickness); // top horizontal secion
    spawn_wall(commands, arena_materials, left, bottom, utils::WIN_W, wall_thickness); // bottom horizontal secion

    let mut y = top - wall_thickness;
    spawn_wall(commands, arena_materials, left, y, wall_thickness, vertical_section_size); // left upper section above goalpost
    y -= vertical_section_size;
    spawn_goal_post(commands, arena_materials, team::Team::Home, left, y, wall_thickness, goal_post_size);
    y -= goal_post_size;
    spawn_wall(commands, arena_materials, left, y, wall_thickness, vertical_section_size); // left lower section below goalpost

    y = top - wall_thickness;
    spawn_wall(commands, arena_materials, right, y, wall_thickness, vertical_section_size); // right upper section above goalpost
    y -= vertical_section_size;
    spawn_goal_post(commands, arena_materials, team::Team::Away, right, y, wall_thickness, goal_post_size);
    y -= goal_post_size;
    spawn_wall(commands, arena_materials, right, y, wall_thickness, vertical_section_size); // right lower section below goalpost
}