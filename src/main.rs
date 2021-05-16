use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RapierConfiguration, RapierPhysicsPlugin},
    rapier::math::Vector,
};

mod modules;
use modules::{
    ui,
    ball,
    player,
    input,
    utils,
    helpers,
    states,
    round,
    collision,
    physics,
    animation,
};




fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut configuration: ResMut<RapierConfiguration>,
) {
    configuration.gravity = Vector::y() * 0.0;
    configuration.time_dependent_number_of_timesteps = true;

    helpers::setup_helper_materials(&mut commands, &asset_server, &mut materials);
    player::setup_player_sprites(&mut commands, &asset_server, &mut texture_atlases);
    ui::setup_ui_materials(&mut commands, &asset_server);
    ball::setup_ball_material(&mut commands, &asset_server, &mut texture_atlases);
    commands.insert_resource(player::CurrentControlMode(player::ControlMode::Run));
}

fn initialize_game(
    mut commands: Commands,
    fonts: Res<ui::FontMaterials>,
    helper_materiarls: Res<helpers::HelperMaterials>,
    ball_sprite: Res<ball::BallTexture>,
    player_sprites: Res<player::PlayerTextures>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    ui::spawn_ui(&mut commands, &fonts);
    player::spawn_player(&mut commands, &player_sprites, Vec2::new(100.0, 0.0), true, true);
    player::spawn_player(&mut commands, &player_sprites, Vec2::new(-100.0, 0.0), false, false);
    ball::spawn_ball(&mut commands, &ball_sprite, Vec2::new(0.0, 0.0), Vec2::ZERO, None);
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
        .add_plugin(RapierPhysicsPlugin)
        .add_event::<collision::RRCollisionEvent>()
        .add_event::<ball::BallEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_initialization", SystemStage::single(initialize_game.system()))
        .add_system_set(ui::ui_changes_listeners())
        .add_system_set(
            SystemSet::on_enter(states::AppState::Plan)
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
                .with_system(player::reset_control_mode.system())
                .with_system(physics::resume_physics.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Play)
                .with_system(animation::animate_sprite.system())
                .with_system(round::update_timer.system())
                .with_system(player::handle_players_action_finish.system()
                    .label("handle_players_action_finish")
                )
                .with_system(collision::get_contact_events.system()
                    .label("get_contact_events")
                )
                .with_system(collision::handle_collision_events.system()
                    .label("handle_collision_events")
                    .after("get_contact_events")
                )
                .with_system(ball::update_thrown_ball.system()
                    .label("update_thrown_ball")
                    .after("handle_collision_events")
                )
                .with_system(player::handle_player_action_start.system()
                    .label("handle_player_action_start")
                    .after("handle_players_action_finish")
                    .after("handle_collision_events")
                )
                .with_system(ball::handle_ball_events.system()
                    .label("handle_ball_events")
                    .before("update_thrown_ball")
                    .after("handle_player_action_start"))
                .with_system(player::update_helpers.system()
                    .after("handle_player_action_start")
                    .after("update_thrown_ball")
                )
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Play)
                .with_system(player::reset_move_actions.system().label("reset_movement"))
                .with_system(player::handle_player_action_start.system()
                    .label("actor_state_change")
                    .after("reset_movement")
                )
                .with_system(physics::pause_physics.system()
                    .after("actor_state_change")
                )
        )
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}
