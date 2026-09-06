use super::{GameState, PlayerId};

#[derive(Debug, Clone)]
pub struct RoundState {
    pub round: u32,
    pub scores: [u32; 4],
    pub intermission_seconds: f32,
}

impl RoundState {
    pub fn new() -> Self {
        Self {
            round: 1,
            scores: [0; 4],
            intermission_seconds: 0.0,
        }
    }

    pub fn winner(state: &GameState) -> Option<PlayerId> {
        let alive = state
            .tanks
            .iter()
            .filter(|tank| tank.alive)
            .collect::<Vec<_>>();
        if alive.len() == 1 {
            Some(alive[0].player_id)
        } else {
            None
        }
    }

    pub fn award(&mut self, player_id: PlayerId) {
        let index = player_id.0 as usize;
        if index < self.scores.len() {
            self.scores[index] += 1;
        }
        self.intermission_seconds = 2.5;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.intermission_seconds = (self.intermission_seconds - dt).max(0.0);
        self.intermission_seconds == 0.0
    }

    pub fn next_round(&mut self) {
        self.round += 1;
        self.intermission_seconds = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GameMap, MapSize, Vec2};

    #[test]
    fn winner_is_only_alive_tank() {
        let mut state = GameState::default();
        let a = state.add_player("A", None).unwrap();
        let b = state.add_player("B", None).unwrap();
        state.add_tank(a, Vec2::new(50.0, 50.0));
        state.add_tank(b, Vec2::new(100.0, 50.0));
        state.tanks[1].destroy();
        assert_eq!(RoundState::winner(&state), Some(a));
        let _ = GameMap::generate(MapSize::Small);
    }

    #[test]
    fn score_and_round_advance() {
        let mut round = RoundState::new();
        round.award(PlayerId(2));
        assert_eq!(round.scores[2], 1);
        assert!(!round.tick(1.0));
        assert!(round.tick(2.0));
        round.next_round();
        assert_eq!(round.round, 2);
    }
}
