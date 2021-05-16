use bevy::prelude::*;
use super::{
    states,
};


pub struct RoundTimer {}

const ROUND_TIME: f32 = 2.0;

pub fn start_timer(mut commands: Commands, query: Query<Entity, With<RoundTimer>>) {
    for timer in query.iter() {
        commands.entity(timer).despawn();
    }

    commands
        .spawn()
        .insert(Timer::from_seconds(ROUND_TIME, false))
        .insert(RoundTimer {});
}

pub fn update_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Timer), With<RoundTimer>>,
    mut app_state: ResMut<State<states::AppState>>
) {
    let timer_result = query.single_mut();
    if timer_result.is_err() {
        return;
    }
    let (timer_entity, mut timer) = timer_result.unwrap();
    timer.tick(time.delta());
    if timer.finished() {
        commands.entity(timer_entity).despawn();
        app_state.set(states::AppState::Plan).unwrap();
    }
}
