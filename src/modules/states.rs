#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Plan,
    Play,
    Introduction,
    Scored,
    MovingToStartPosition,
}
