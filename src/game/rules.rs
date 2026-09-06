use super::{GameConfig, GameState, PlayerId, TeamId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    FreeForAll,
    TeamBattle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRules {
    pub mode: GameMode,
    pub max_players: usize,
    pub team_count: usize,
}

impl GameRules {
    pub const fn single_player(mode: GameMode) -> Self {
        Self {
            mode,
            max_players: 4,
            team_count: if matches!(mode, GameMode::TeamBattle) {
                2
            } else {
                0
            },
        }
    }

    pub const fn lan(mode: GameMode) -> Self {
        Self {
            mode,
            max_players: 10,
            team_count: if matches!(mode, GameMode::TeamBattle) {
                2
            } else {
                0
            },
        }
    }

    pub fn with_team_count(mut self, team_count: usize) -> Option<Self> {
        if !matches!(self.mode, GameMode::TeamBattle)
            || !(2..=self.max_players).contains(&team_count)
        {
            return None;
        }
        self.team_count = team_count;
        Some(self)
    }

    pub fn can_fire(state: &GameState, player_id: PlayerId, config: &GameConfig) -> bool {
        state
            .tanks
            .iter()
            .any(|tank| tank.player_id == player_id && tank.alive)
            && state
                .projectiles
                .iter()
                .filter(|p| p.owner == player_id)
                .count()
                < config.max_active_projectiles_per_tank
    }

    pub fn winner(&self, state: &GameState) -> Option<Vec<TeamId>> {
        let alive: Vec<PlayerId> = state
            .tanks
            .iter()
            .filter(|tank| tank.alive)
            .map(|tank| tank.player_id)
            .collect();
        if alive.is_empty() {
            return None;
        }
        match self.mode {
            GameMode::FreeForAll => {
                if alive.len() == 1 {
                    Some(vec![])
                } else {
                    None
                }
            }
            GameMode::TeamBattle => {
                let mut teams = Vec::new();
                for player_id in alive {
                    if let Some(team) = state
                        .players
                        .iter()
                        .find(|player| player.id == player_id)
                        .and_then(|p| p.team)
                    {
                        if !teams.contains(&team) {
                            teams.push(team);
                        }
                    }
                }
                if teams.len() == 1 {
                    Some(teams)
                } else {
                    None
                }
            }
        }
    }

    pub fn assign_balanced_team(player_index: usize, team_count: usize) -> Option<TeamId> {
        if team_count < 2 {
            None
        } else {
            Some(TeamId((player_index % team_count) as u8))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_player_has_four_player_limit() {
        assert_eq!(
            GameRules::single_player(GameMode::FreeForAll).max_players,
            4
        );
    }

    #[test]
    fn lan_has_ten_player_limit() {
        assert_eq!(GameRules::lan(GameMode::TeamBattle).max_players, 10);
    }

    #[test]
    fn team_count_is_validated() {
        let rules = GameRules::lan(GameMode::TeamBattle);
        assert!(rules.with_team_count(2).is_some());
        assert!(rules.with_team_count(10).is_some());
        assert!(rules.with_team_count(1).is_none());
    }

    #[test]
    fn teams_are_assigned_balanced() {
        assert_eq!(GameRules::assign_balanced_team(0, 2), Some(TeamId(0)));
        assert_eq!(GameRules::assign_balanced_team(1, 2), Some(TeamId(1)));
        assert_eq!(GameRules::assign_balanced_team(2, 2), Some(TeamId(0)));
    }

    #[test]
    fn ffa_winner_requires_one_survivor() {
        let rules = GameRules::single_player(GameMode::FreeForAll);
        let mut state = GameState::default();
        let a = state.add_player("A", None).unwrap();
        let b = state.add_player("B", None).unwrap();
        state.add_tank(a, super::super::Vec2::ZERO);
        state.add_tank(b, super::super::Vec2::new(50.0, 0.0));
        assert!(rules.winner(&state).is_none());
        state.destroy_tank(b);
        assert_eq!(rules.winner(&state), Some(vec![]));
    }
}
