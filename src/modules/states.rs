use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Plan,
    Play,
}

pub fn state_change(
    mut query_text: Query<&mut Text, With<super::ui::StateText>>,
    app_state: Res<State<super::states::AppState>>,
) {
    let mut text = query_text.single_mut().expect("Cannot access Diagnostic Text");
    super::ui::update_text(&mut text, format!("State: {:?}", app_state.current()));
}
