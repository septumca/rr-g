use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RapierConfiguration, RapierPhysicsPlugin},
    rapier::{math::Vector},
    // render::RapierRenderPlugin,
};

mod modules;
use modules::{actor, animation, arena, ball, collision, helpers, input, matchup, physics, round, states, team, ui, utils};


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
    commands.insert_resource(matchup::Matchup::new(Vec2::new(-100.0, 0.0), Vec2::new(100.0, 0.0)));
}

fn initialize_game(
    mut commands: Commands,
    fonts: Res<ui::FontMaterials>,
    helper_materiarls: Res<helpers::HelperMaterials>,
    arena_materials: Res<arena::ArenaMaterials>,
    actor_sprites: Res<actor::ActorTextures>,
    mut matchup_res: ResMut<matchup::Matchup>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let ui_size = 20.0;

    ui::spawn_ui(&mut commands, &fonts);

    let actors: Vec<(Entity, Vec2, team::Team)> = vec![
        (Vec2::new(-150.0, 100.0), Vec2::new(-150.0, 0.0),  team::Team::Home),
        (Vec2::new(-100.0, 100.0), Vec2::new(-100.0, -100.0),  team::Team::Home),
        (Vec2::new(-50.0, 100.0), Vec2::new(-100.0, 100.0),  team::Team::Home),
        (Vec2::new(150.0, 100.0), Vec2::new(150.0, 0.0),  team::Team::Away),
        (Vec2::new(100.0, 100.0), Vec2::new(100.0, -100.0),  team::Team::Away),
        (Vec2::new(50.0, 100.0), Vec2::new(100.0, 100.0),  team::Team::Away),
    ].iter().map(|(initial_position, target_position, team)| -> (Entity, Vec2, team::Team) {
        (
            actor::spawn_actor(&mut commands, &actor_sprites, *initial_position, *team),
            target_position.clone(),
            team.clone()
        )
    }).collect();

    matchup_res.add_actors(actors);
    arena::create_simple(&mut commands, &arena_materials, utils::WIN_W, utils::WIN_H - ui_size, 0.0, ui_size);
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
        .add_state(states::AppState::Introduction)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        // .add_plugin(RapierRenderPlugin)
        .add_event::<collision::RRCollisionEvent>()
        .add_event::<ball::BallEvent>()
        .add_event::<actor::ActorEvents>()
        .add_event::<matchup::MatchupEvents>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_initialization", SystemStage::single(initialize_game.system()))
        .add_system_set(ui::ui_changes_listeners())
        .add_system(animation::animate_sprite.system())
        .add_system_set(
            SystemSet::on_enter(states::AppState::Introduction)
                .with_system(ui::spawn_score_text.system())
                .with_system(ui::add_pre_game_text.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Introduction)
                .with_system(input::handle_keyboard_input_pre_round.system())
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Introduction)
                .with_system(ui::clear_game_text.system())
        )
        .add_system_set(
            SystemSet::on_enter(states::AppState::Scored)
                .with_system(ui::add_score_text.system())
                .with_system(physics::pause_physics.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::Scored)
                .with_system(input::handle_keyboard_input_pre_round.system())
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Scored)
                .with_system(ui::clear_game_text.system())
        )
        .add_system_set(
            SystemSet::on_enter(states::AppState::MovingToStartPosition)
                .with_system(physics::resume_physics.system())
                .with_system(matchup::move_actors_to_positions.system())
        )
        .add_system_set(
            SystemSet::on_update(states::AppState::MovingToStartPosition)
                .with_system(actor::handle_actors_refresh_action.system()
                    .label("handle_actors_refresh_action_start_position")
                )
                .with_system(actor::handle_actor_action_start.system()
                    .label("handle_actor_action_start_start_position")
                    .after("handle_actors_refresh_action_start_position")
                )
                .with_system(matchup::are_actors_in_position.system()
                    .after("handle_actor_action_start_start_position")
                )
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::MovingToStartPosition)
                .with_system(matchup::update_actors_facing.system())
                .with_system(ball::add_ball_to_arena.system())
        )
        .add_system_set(
            SystemSet::on_enter(states::AppState::Plan)
                .with_system(helpers::cleanup_movement_helpers.system())
                .with_system(physics::pause_physics.system())
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
                .with_system(actor::handle_actor_events.system()
                    .label("handle_actor_events")
                    .after("handle_actors_refresh_action")
                )
                .with_system(ball::handle_ball_events.system()
                    .label("handle_ball_events")
                    .after("handle_actor_events")
                )
                .with_system(ball::update_thrown_ball.system()
                    .label("update_thrown_ball")
                    .after("handle_ball_events")
                )
                .with_system(matchup::handle_matchup_events.system()
                    .label("handle_matchup_events")
                    .after("update_thrown_ball")
                )
                .with_system(actor::update_helpers.system()
                    .after("handle_matchup_events")
                )
        )
        .add_system_set(
            SystemSet::on_exit(states::AppState::Play)
                .with_system(actor::after_round_reset.system().label("after_round_reset"))
                .with_system(actor::handle_actor_action_start.system()
                    .after("after_round_reset")
                )
        )
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}
