use bevy::prelude::*;
mod modules;

use modules::{ui, player, input, utils};

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("rr2.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(8.0, 8.0), 6, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());


    ui::spawn_ui(&mut commands, &asset_server);
    player::spawn_player(&texture_atlas_handle, &mut commands, Vec3::new(-150.0, 0.0, 1.0));
    player::spawn_player(&texture_atlas_handle, &mut commands, Vec3::new(-150.0, 50.0, 1.0));
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
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(player::animate_sprite.system())
        .add_system(player::player_movement.system())
        .add_system(input::handle_mouse_click.system())
        .run();
}