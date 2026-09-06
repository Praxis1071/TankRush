use super::{GameConfig, GameState, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    FreeForAll,
    TeamBattle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRules {
    pub mode: GameMode,
    pub max_players: usize,
}

impl GameRules {
    pub const fn single_player(mode: GameMode) -> Self {
        Self { mode, max_players: 4 }
    }

    pub const fn lan(mode: GameMode) -> Self {
        Self { mode, max_players: 10 }
    }

    pub fn can_fire(state: &GameState, player_id: PlayerId, config: &GameConfig) -> bool {
        state.projectiles.iter().filter(|p| p.owner == player_id).count() < config.max_active_projectiles_per_tank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_player_has_four_player_limit() {
        assert_eq!(GameRules::single_player(GameMode::FreeForAll).max_players, 4);
    }

    #[test]
    fn lan_has_ten_player_limit() {
        assert_eq!(GameRules::lan(GameMode::TeamBattle).max_players, 10);
    }
}
