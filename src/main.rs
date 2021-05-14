use bevy::prelude::*;
use heron::prelude::*;

mod modules;
use modules::{ui, player, input, utils, helpers, states, round};




fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    player::spawn_player(&mut commands, &player_sprites, Vec3::new(0.0, 0.0, 1.0));
    player::spawn_player(&mut commands, &player_sprites, Vec3::new(-100.0, -100.0, 1.0));
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
        .add_state(states::AppState::Plan)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_startup_system(setup.system())
        .add_startup_stage("game_initialization", SystemStage::single(initialize_game.system()))
        .add_system_set(
            SystemSet::on_enter(states::AppState::Plan)
                .with_system(states::state_change.system())
                .with_system(helpers::cleanup_movement_helpers.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Plan)
                .with_system(input::handle_mouse_click.system())
                .with_system(input::handle_keyboard_input.system())
                .with_system(helpers::update_selected_helper.system())
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Plan)
                .with_system(helpers::deselect_all.system())
        )
        .add_system_set(
            SystemSet::on_enter(states::AppState::Play)
                .with_system(round::start_timer.system())
                .with_system(states::state_change.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Play)
                .with_system(player::animate_sprite.system())
                .with_system(round::update_timer.system())
                .with_system(player::update_players_actions.system().label("actions_update_step1"))
                .with_system(player::handle_player_action_change.system()
                    .label("actions_update_step2")
                    .after("actions_update_step1")
                )
                .with_system(player::update_helpers.system().after("actions_update_step2"))
                .with_system(player::handle_collisions.system().before("actions_update_step2"))
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Play)
                .with_system(player::reset_move_actions.system().label("reset_movement"))
                .with_system(player::handle_player_action_change.system().after("reset_movement"))
        )
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}
