use crate::game::{GameMode, PlayerId, TeamId};

pub const MAX_LOBBY_PLAYERS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LobbyPlayer {
    pub id: PlayerId,
    pub name: String,
    pub ready: bool,
    pub team: Option<TeamId>,
}

#[derive(Debug, Clone)]
pub struct LobbyState {
    pub mode: GameMode,
    pub players: Vec<LobbyPlayer>,
    pub team_count: usize,
}

impl LobbyState {
    pub fn new(mode: GameMode, team_count: usize) -> Option<Self> {
        if matches!(mode, GameMode::TeamBattle)
            && !(2..=MAX_LOBBY_PLAYERS).contains(&team_count)
        {
            return None;
        }
        Some(Self {
            mode,
            players: Vec::new(),
            team_count: if matches!(mode, GameMode::TeamBattle) {
                team_count
            } else {
                0
            },
        })
    }

    pub fn join(&mut self, name: impl Into<String>) -> Option<PlayerId> {
        if self.players.len() >= MAX_LOBBY_PLAYERS {
            return None;
        }
        let id = PlayerId(self.players.len() as u8);
        let team = if matches!(self.mode, GameMode::TeamBattle) {
            Some(TeamId((self.players.len() % self.team_count) as u8))
        } else {
            None
        };
        self.players.push(LobbyPlayer {
            id,
            name: name.into(),
            ready: false,
            team,
        });
        Some(id)
    }

    pub fn leave(&mut self, player_id: PlayerId) -> bool {
        let old_len = self.players.len();
        self.players.retain(|player| player.id != player_id);
        self.reassign_teams();
        old_len != self.players.len()
    }

    pub fn set_ready(&mut self, player_id: PlayerId, ready: bool) -> bool {
        if let Some(player) = self.players.iter_mut().find(|player| player.id == player_id) {
            player.ready = ready;
            true
        } else {
            false
        }
    }

    pub fn all_ready(&self) -> bool {
        !self.players.is_empty() && self.players.iter().all(|player| player.ready)
    }

    pub fn team_members(&self, team: TeamId) -> usize {
        self.players
            .iter()
            .filter(|player| player.team == Some(team))
            .count()
    }

    fn reassign_teams(&mut self) {
        if !matches!(self.mode, GameMode::TeamBattle) {
            return;
        }
        for (index, player) in self.players.iter_mut().enumerate() {
            player.team = Some(TeamId((index % self.team_count) as u8));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lobby_caps_at_ten_players() {
        let mut lobby = LobbyState::new(GameMode::FreeForAll, 0).unwrap();
        for index in 0..10 {
            assert!(lobby.join(format!("P{index}")).is_some());
        }
        assert!(lobby.join("P10").is_none());
    }

    #[test]
    fn ready_state_requires_every_player() {
        let mut lobby = LobbyState::new(GameMode::FreeForAll, 0).unwrap();
        let player = lobby.join("P1").unwrap();
        assert!(!lobby.all_ready());
        lobby.set_ready(player, true);
        assert!(lobby.all_ready());
    }

    #[test]
    fn team_assignment_stays_balanced() {
        let mut lobby = LobbyState::new(GameMode::TeamBattle, 2).unwrap();
        for index in 0..4 {
            lobby.join(format!("P{index}"));
        }
        assert_eq!(lobby.team_members(TeamId(0)), 2);
        assert_eq!(lobby.team_members(TeamId(1)), 2);
    }

    #[test]
    fn leaving_rebalances_teams() {
        let mut lobby = LobbyState::new(GameMode::TeamBattle, 2).unwrap();
        let first = lobby.join("P1").unwrap();
        lobby.join("P2");
        lobby.join("P3");
        lobby.leave(first);
        assert_eq!(lobby.team_members(TeamId(0)), 1);
        assert_eq!(lobby.team_members(TeamId(1)), 1);
    }
}
