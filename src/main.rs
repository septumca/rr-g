use bevy::{input::mouse::{MouseButtonInput, MouseMotion, MouseWheel}, prelude::*, transform, window::CursorMoved};

const WIN_W: f32 = 800.0;
const WIN_H: f32 = 600.0;

struct DiagText;

type AnimationTimer = Timer;
struct Animation {
    act_frame_index: usize,
    sprite_indexes: Vec<usize>
}

type TargetPosition = Vec3;


struct Actor {}

enum ActorState {
    Idle,
    Running,
    Diving,
    Recovering
}
struct Selected {}
// struct Selected {
//     entity: Option<Entity>
// }
// impl SelectedActor {
//     pub fn is_entity_selected(&self, entity: &Entity) -> bool {
//         if let Some(e) = self.entity {
//             return e == *entity
//         }
//         false
//     }

//     pub fn select_entity(&mut self, entity: Entity) {
//         self.entity = Some(entity);
//     }
// }

impl Animation {
    pub fn new(sprite_indexes: Vec<usize>) -> Self {
        Self {
            act_frame_index: 0,
            sprite_indexes
        }
    }

    pub fn update(&mut self) {
        self.act_frame_index = (self.act_frame_index + 1) % self.sprite_indexes.len()
    }

    pub fn get_sprite_index(&self) -> u32 {
        return self.sprite_indexes[self.act_frame_index] as u32;
    }
}

fn spawn_player(texture_atlas_handle: &Handle<TextureAtlas>, commands: &mut Commands, position: Vec3) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform {
                rotation: Quat::IDENTITY,
                translation: position,
                scale: Vec3::splat(4.0)
            },
            ..Default::default()
        })
        .insert(Actor {})
        .insert(ActorState::Idle)
        // .insert(TargetPosition::new(150.0, 0.0, 1.0))
        .insert(Animation::new(vec![0]))
        .insert(AnimationTimer::from_seconds(1.0/8.0, true));
}

fn spawn_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font_path = "FiraCode-Medium.ttf";
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "No entity selected".to_string(),
                        style: TextStyle {
                            font: asset_server.load(font_path.clone()),
                            font_size: 10.0,
                            color: Color::WHITE,
                        },
                    }
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(DiagText);
}

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


    spawn_ui(&mut commands, &asset_server);
    spawn_player(&texture_atlas_handle, &mut commands, Vec3::new(-150.0, 0.0, 1.0));
    spawn_player(&texture_atlas_handle, &mut commands, Vec3::new(-150.0, 50.0, 1.0));
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, &mut Animation)>,
) {
    for (mut timer, mut sprite, mut animation) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            animation.update();
            sprite.index = animation.get_sprite_index();
        }
    }
}

fn transform_pos_window_to_screen(window_pos: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    Vec2::new(window_pos.x - WIN_W / 2.0, window_pos.y - WIN_H / 2.0)
}

fn is_actor_clicked(player_pos: &bevy::prelude::Vec3, click_pos: bevy::prelude::Vec2) -> bool {
    click_pos.x > (player_pos.x - 16.0) &&
        click_pos.x < (player_pos.x + 16.0) &&
        click_pos.y > (player_pos.y - 16.0) &&
        click_pos.y < (player_pos.y + 16.0)
}

fn handle_mouse_clicks(
    mut query: Query<(Entity, &Transform), (With<Actor>, Without<Selected>)>,
    mut query_selected: Query<(Entity, &mut Animation), (With<Actor>, With<Selected>)>,
    mut query_text: Query<&mut Text, With<DiagText>>,
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>
) {
    let click_pos = windows
        .get_primary()
        .and_then(|win| -> Option<bevy::prelude::Vec2> {
            if !mouse_input.just_pressed(MouseButton::Left) {
                return None;
            }
            win.cursor_position()
        })
        .and_then(|pos| -> Option<bevy::prelude::Vec2> {
            Some(transform_pos_window_to_screen(pos))
        });
    if click_pos.is_none() {
        return;
    }
    let click_pos = click_pos.unwrap();

    let mut clicked_entity = None;
    for (entity, transform) in query.iter_mut() {
        println!("Checking clicked: {:?} -> {} ? {}", entity, transform.translation, click_pos);
        if is_actor_clicked(&transform.translation, click_pos) {
            clicked_entity = Some(entity)
        }
    }

    if clicked_entity.is_some() {
        for (prev_selected, _) in query_selected.iter_mut() {
            commands.entity(prev_selected).remove::<Selected> ();
        }
        let clicked_entity = clicked_entity.unwrap();
        for mut text in query_text.iter_mut() {
            text.sections[0].value = format!("Selected Entity: {:?}", clicked_entity.clone());
        }
        commands.entity(clicked_entity).insert(Selected {});
    } else {
        for (selected, mut animation) in query_selected.iter_mut() {
            commands.entity(selected).insert(TargetPosition::new(click_pos.x, click_pos.y, 1.0));
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0, 1, 0, 2];
        }
    }
}

fn player_movement(mut query: Query<(Entity, &TargetPosition, &mut Transform, &mut Animation, &mut TextureAtlasSprite), With<Actor>>, mut commands: Commands) {
    for (entity, target_position, mut transform, mut animation, mut sprite) in query.iter_mut() {
        if transform.translation.x == target_position.x && transform.translation.y == target_position.y {
            commands.entity(entity).remove::<TargetPosition> ();
            animation.act_frame_index = 0;
            animation.sprite_indexes = vec![0];
            return;
        }

        let delta = *target_position - transform.translation;
        sprite.flip_x = delta.x < 0.0;

        if delta.x.abs() < 1.0 && delta.y.abs() < 1.0 {
            transform.translation.x = target_position.x;
            transform.translation.y = target_position.y;
            return;
        }

        let delta = delta.normalize();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Lobda".to_string(),
            width: WIN_W,
            height: WIN_H,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(animate_sprite_system.system())
        .add_system(player_movement.system())
        .add_system(handle_mouse_clicks.system())
        .run();
}