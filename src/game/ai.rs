use super::{GameState, PlayerId, TankInput, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct AiController {
    pub target: Option<PlayerId>,
    pub fire_cooldown: f32,
    pub think_timer: f32,
}

impl Default for AiController {
    fn default() -> Self {
        Self {
            target: None,
            fire_cooldown: 0.0,
            think_timer: 0.0,
        }
    }
}

impl AiController {
    pub fn input(&mut self, state: &GameState, self_id: PlayerId, dt: f32) -> TankInput {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        self.think_timer -= dt;
        if self.think_timer <= 0.0 {
            self.target = nearest_enemy(state, self_id);
            self.think_timer = 0.12;
        }
        let Some(tank) = state
            .tanks
            .iter()
            .find(|t| t.player_id == self_id && t.alive)
        else {
            return TankInput::idle();
        };
        let Some(target_id) = self.target else {
            return TankInput::idle();
        };
        let Some(target) = state
            .tanks
            .iter()
            .find(|t| t.player_id == target_id && t.alive)
        else {
            return TankInput::idle();
        };
        let to_target = target.position - tank.position;
        let desired = to_target.y.atan2(to_target.x);
        let mut delta = desired - tank.rotation_radians;
        while delta > std::f32::consts::PI {
            delta -= std::f32::consts::TAU;
        }
        while delta < -std::f32::consts::PI {
            delta += std::f32::consts::TAU;
        }
        let aligned = delta.abs() < 0.18;
        let distance = to_target.length();
        TankInput {
            forward: distance > 180.0 || !aligned,
            backward: distance < 90.0 && aligned,
            left: delta < -0.06,
            right: delta > 0.06,
            fire: aligned && self.fire_cooldown <= 0.0,
        }
        .tap_fire(self)
    }
}

trait TapFire {
    fn tap_fire(self, controller: &mut AiController) -> Self;
}
impl TapFire for TankInput {
    fn tap_fire(self, controller: &mut AiController) -> Self {
        if self.fire {
            controller.fire_cooldown = 0.45;
        }
        self
    }
}

fn nearest_enemy(state: &GameState, self_id: PlayerId) -> Option<PlayerId> {
    let origin = state
        .tanks
        .iter()
        .find(|tank| tank.player_id == self_id && tank.alive)?
        .position;
    state
        .tanks
        .iter()
        .filter(|tank| tank.alive && tank.player_id != self_id)
        .map(|tank| ((tank.position - origin).length(), tank.player_id))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GameConfig, GameMap, GameSimulation, MapSize};
    #[test]
    fn ai_targets_another_alive_tank() {
        let mut sim = GameSimulation::new(GameMap::generate(MapSize::Small), GameConfig::default());
        let ai = sim.state.add_player("AI", None).unwrap();
        let enemy = sim.state.add_player("Enemy", None).unwrap();
        sim.state.add_tank(ai, Vec2::new(100.0, 100.0));
        sim.state.add_tank(enemy, Vec2::new(200.0, 100.0));
        let mut controller = AiController::default();
        let input = controller.input(&sim.state, ai, 1.0 / 60.0);
        assert!(input.right || input.forward || input.fire);
        assert_eq!(controller.target, Some(enemy));
    }
}
