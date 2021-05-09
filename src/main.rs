use bevy::prelude::*;

type AnimationTimer = Timer;
struct Animation {
    act_frame_index: usize,
    sprite_indexes: Vec<usize>
}

type TargetPosition = Vec3;

struct Player {}

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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("rr2.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(8.0, 8.0), 6, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                rotation: Quat::IDENTITY,
                translation: Vec3::new(-150.0, 0.0, 1.0),
                scale: Vec3::splat(4.0)
            },
            ..Default::default()
        })
        .insert(Player {})
        .insert(TargetPosition::new(150.0, 0.0, 1.0))
        .insert(Animation::new(vec![0, 1, 0, 2]))
        .insert(AnimationTimer::from_seconds(1.0/8.0, true));
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

fn player_movement(mut query: Query<(&Player, &TargetPosition, &mut Transform)>) {
    for (_, target_position, mut transform) in query.iter_mut() {
        if transform.translation.x == target_position.x && transform.translation.y == target_position.y {
            return;
        }

        let delta = *target_position - transform.translation;
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
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(animate_sprite_system.system())
        .add_system(player_movement.system())
        .run();
}