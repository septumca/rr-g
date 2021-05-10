use bevy::prelude::*;
mod modules;

use modules::{ui, player, input, utils, helpers};

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    helpers::setup_helper_materials(&mut commands, &asset_server, &mut materials);
    player::setup_player_sprites(&mut commands, &asset_server, texture_atlases);
    ui::setup_ui_materials(&mut commands, &asset_server);
}

fn initialize_game(
    mut commands: Commands,
    fonts: Res<ui::FontMaterials>,
    helper_materiarls: Res<helpers::HelperMaterials>,
    player_sprites: Res<player::PlayerTextures>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    ui::spawn_ui(&mut commands, &fonts);
    player::spawn_player(&mut commands, &player_sprites, Vec3::new(-150.0, 0.0, 1.0));
    player::spawn_player(&mut commands, &player_sprites, Vec3::new(-150.0, 50.0, 1.0));
    helpers::spawn_selected_helper(&mut commands, &helper_materiarls);
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Lobda".to_string(),
            width: utils::WIN_W,
            height: utils::WIN_H,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage("game_initialization", SystemStage::single(initialize_game.system()))
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(player::animate_sprite.system())
        .add_system(player::player_movement.system())
        .add_system(input::handle_mouse_click.system())
        .add_system(helpers::update_selected_helper.system())
        .run();
}