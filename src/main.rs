use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RapierConfiguration, RapierPhysicsPlugin},
    rapier::{math::Vector},
    // render::RapierRenderPlugin,
};

mod modules;
use modules::{
    arena,
    ui,
    ball,
    actor,
    input,
    utils,
    helpers,
    states,
    round,
    collision,
    physics,
    animation,
    team,
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
    actor::setup_actor_sprites(&mut commands, &asset_server, &mut texture_atlases);
    ui::setup_ui_materials(&mut commands, &asset_server);
    ball::setup_ball_material(&mut commands, &asset_server, &mut texture_atlases);
    arena::setup_arena_materials(&mut commands, &mut materials);
    commands.insert_resource(actor::CurrentControlMode(actor::ControlMode::Run));
}

fn initialize_game(
    mut commands: Commands,
    fonts: Res<ui::FontMaterials>,
    helper_materiarls: Res<helpers::HelperMaterials>,
    arena_materials: Res<arena::ArenaMaterials>,
    ball_sprite: Res<ball::BallTexture>,
    actor_sprites: Res<actor::ActorTextures>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let ui_size = 20.0;

    ui::spawn_ui(&mut commands, &fonts);
    actor::spawn_actor(&mut commands, &actor_sprites, Vec2::new(100.0, 100.0), team::Team::Away);
    actor::spawn_actor(&mut commands, &actor_sprites, Vec2::new(100.0, -100.0), team::Team::Away);
    actor::spawn_actor(&mut commands, &actor_sprites, Vec2::new(-100.0, 100.0), team::Team::Home);
    actor::spawn_actor(&mut commands, &actor_sprites, Vec2::new(-100.0, -100.0), team::Team::Home);
    arena::create_simple(&mut commands, &arena_materials, utils::WIN_W, utils::WIN_H - ui_size, 0.0, ui_size);
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
        // .add_plugin(RapierRenderPlugin)
        .add_event::<collision::RRCollisionEvent>()
        .add_event::<ball::BallEvent>()
        .add_event::<actor::ActorEvents>()
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
                .with_system(actor::reset_control_mode.system())
                .with_system(physics::resume_physics.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Play)
                .with_system(animation::animate_sprite.system())
                .with_system(round::update_timer.system())
                .with_system(collision::get_contact_events.system()
                    .label("get_contact_events")
                )
                .with_system(collision::handle_collision_events.system()
                    .label("handle_collision_events")
                    .after("get_contact_events")
                )
                .with_system(actor::handle_actor_action_start.system()
                    .label("handle_actor_action_start")
                    .after("handle_collision_events")
                )
                .with_system(actor::handle_actors_refresh_action.system()
                    .label("handle_actors_refresh_action")
                    .after("handle_actor_action_start")
                )
                .with_system(actor::handle_player_events.system()
                    .label("handle_player_events")
                    .after("handle_actors_refresh_action")
                )
                .with_system(ball::handle_ball_events.system()
                    .label("handle_ball_events")
                    .after("handle_player_events")
                )
                .with_system(ball::update_thrown_ball.system()
                    .label("update_thrown_ball")
                    .after("handle_ball_events")
                )
                .with_system(actor::update_helpers.system()
                    .after("update_thrown_ball")
                )
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Play)
                .with_system(actor::after_round_reset.system().label("after_round_reset"))
                .with_system(actor::handle_actor_action_start.system()
                    .label("actor_state_change")
                    .after("after_round_reset")
                )
                .with_system(physics::pause_physics.system()
                    .after("actor_state_change")
                )
        )
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}
