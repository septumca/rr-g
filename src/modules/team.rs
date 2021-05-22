#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Team {
    Home,
    Away
}

pub fn get_oposing_team(team: Team) -> Team {
    match team {
        Team::Away => Team::Home,
        Team::Home => Team::Away,
    }
}