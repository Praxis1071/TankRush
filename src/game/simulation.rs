use super::{GameConfig, GameMap, GameState, PlayerId, Vec2, collision::segment_wall_hit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TankInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool,
}

impl TankInput {
    pub const fn idle() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            fire: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationEvent {
    pub player_id: PlayerId,
    pub kind: SimulationEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationEventKind {
    Fired,
    TankDestroyed,
    ProjectileBounced,
}

#[derive(Debug, Clone)]
pub struct GameSimulation {
    pub state: GameState,
    pub map: GameMap,
    pub config: GameConfig,
    accumulator: f32,
}

impl GameSimulation {
    pub fn new(map: GameMap, config: GameConfig) -> Self {
        Self {
            state: GameState::default(),
            map,
            config,
            accumulator: 0.0,
        }
    }

    pub fn advance(
        &mut self,
        elapsed_seconds: f32,
        inputs: &[(PlayerId, TankInput)],
    ) -> Vec<SimulationEvent> {
        self.accumulator += elapsed_seconds.clamp(0.0, 0.25);
        let mut events = Vec::new();
        while self.accumulator >= self.config.fixed_timestep_seconds {
            events.extend(self.step(inputs));
            self.accumulator -= self.config.fixed_timestep_seconds;
        }
        events
    }

    pub fn step(&mut self, inputs: &[(PlayerId, TankInput)]) -> Vec<SimulationEvent> {
        let dt = self.config.fixed_timestep_seconds;
        let mut events = Vec::new();

        for &(player_id, input) in inputs {
            let Some(index) = self
                .state
                .tanks
                .iter()
                .position(|tank| tank.player_id == player_id && tank.alive)
            else {
                continue;
            };

            let tank = self.state.tanks[index];
            let mut rotation = tank.rotation_radians;
            if input.left {
                rotation -= self.config.tank_turn_speed_radians * dt;
            }
            if input.right {
                rotation += self.config.tank_turn_speed_radians * dt;
            }
            let direction = Vec2::new(rotation.cos(), rotation.sin());
            let movement_scale = match (input.forward, input.backward) {
                (true, false) => 1.0,
                (false, true) => -1.0,
                _ => 0.0,
            };
            let movement = direction * (self.config.tank_speed * dt * movement_scale);
            let old_position = tank.position;
            let candidate = old_position + movement;
            let new_position = if self.can_place_tank(candidate) {
                candidate
            } else {
                let x_candidate = Vec2::new(candidate.x, old_position.y);
                let y_candidate = Vec2::new(old_position.x, candidate.y);
                if self.can_place_tank(x_candidate) {
                    x_candidate
                } else if self.can_place_tank(y_candidate) {
                    y_candidate
                } else {
                    old_position
                }
            };

            let tank = &mut self.state.tanks[index];
            tank.rotation_radians = rotation;
            tank.position = new_position;

            if input.fire && self.state.fire(player_id, &self.config).is_some() {
                events.push(SimulationEvent {
                    player_id,
                    kind: SimulationEventKind::Fired,
                });
            }
        }

        let mut next_projectiles = Vec::with_capacity(self.state.projectiles.len());
        let projectiles = std::mem::take(&mut self.state.projectiles);
        for mut projectile in projectiles {
            if projectile.expired() {
                continue;
            }

            let start = projectile.position;
            let end = start + projectile.velocity * dt;
            let wall_hit = self
                .map
                .walls
                .iter()
                .filter_map(|wall| segment_wall_hit(start, end, *wall))
                .min_by(|a, b| a.distance.total_cmp(&b.distance));

            if let Some(hit) = wall_hit {
                projectile.position =
                    hit.point + hit.normal * (self.config.projectile_radius + 0.01);
                projectile.bounce(hit.normal);
                projectile.remaining_lifetime -= dt;
                events.push(SimulationEvent {
                    player_id: projectile.owner,
                    kind: SimulationEventKind::ProjectileBounced,
                });
            } else {
                projectile.position = end;
                projectile.remaining_lifetime -= dt;
            }

            if let Some(target_id) = self.hit_tank(projectile.owner, start, projectile.position) {
                if self.state.destroy_tank(target_id) {
                    events.push(SimulationEvent {
                        player_id: target_id,
                        kind: SimulationEventKind::TankDestroyed,
                    });
                }
                continue;
            }

            if !projectile.expired() {
                next_projectiles.push(projectile);
            }
        }
        self.state.projectiles = next_projectiles;
        events
    }

    fn can_place_tank(&self, position: Vec2) -> bool {
        self.map
            .is_inside_play_area(position, self.config.tank_radius)
            && !self.map.walls.iter().any(|wall| {
                super::collision::circle_intersects_wall(position, self.config.tank_radius, *wall)
            })
    }

    fn hit_tank(&self, owner: PlayerId, start: Vec2, end: Vec2) -> Option<PlayerId> {
        self.state
            .tanks
            .iter()
            .filter(|tank| tank.alive && tank.player_id != owner)
            .filter_map(|tank| {
                segment_circle_hit(start, end, tank.position, self.config.tank_radius)
                    .map(|distance| (distance, tank.player_id))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, player_id)| player_id)
    }
}

fn segment_circle_hit(start: Vec2, end: Vec2, center: Vec2, radius: f32) -> Option<f32> {
    let delta = end - start;
    let offset = start - center;
    let a = delta.dot(delta);
    if a <= f32::EPSILON {
        return (offset.dot(offset) <= radius * radius).then_some(0.0);
    }
    let b = 2.0 * offset.dot(delta);
    let c = offset.dot(offset) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    let t1 = (-b - root) / (2.0 * a);
    let t2 = (-b + root) / (2.0 * a);
    [t1, t2]
        .into_iter()
        .filter(|t| (0.0..=1.0).contains(t))
        .min_by(|a, b| a.total_cmp(b))
        .map(|t| delta.length() * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MapSize;

    #[test]
    fn simulation_uses_fixed_timestep() {
        let mut simulation =
            GameSimulation::new(GameMap::rectangular(MapSize::Small), GameConfig::default());
        let player = simulation.state.add_player("P1", None).unwrap();
        simulation.state.add_tank(player, Vec2::new(100.0, 100.0));
        simulation.advance(
            0.010,
            &[(
                player,
                TankInput {
                    forward: true,
                    ..TankInput::idle()
                },
            )],
        );
        assert_eq!(simulation.state.tanks[0].position, Vec2::new(100.0, 100.0));
        simulation.advance(
            0.010,
            &[(
                player,
                TankInput {
                    forward: true,
                    ..TankInput::idle()
                },
            )],
        );
        assert!(simulation.state.tanks[0].position.x > 100.0);
    }

    #[test]
    fn wall_blocks_tank() {
        let mut simulation =
            GameSimulation::new(GameMap::rectangular(MapSize::Small), GameConfig::default());
        let player = simulation.state.add_player("P1", None).unwrap();
        simulation.state.add_tank(player, Vec2::new(32.0, 100.0));
        simulation.state.tanks[0].rotation_radians = std::f32::consts::PI;
        simulation.advance(
            1.0,
            &[(
                player,
                TankInput {
                    forward: true,
                    ..TankInput::idle()
                },
            )],
        );
        assert!(simulation.state.tanks[0].position.x >= simulation.config.tank_radius + 16.0);
    }

    #[test]
    fn projectile_destroys_other_tank() {
        let mut simulation =
            GameSimulation::new(GameMap::rectangular(MapSize::Small), GameConfig::default());
        let a = simulation.state.add_player("A", None).unwrap();
        let b = simulation.state.add_player("B", None).unwrap();
        simulation.state.add_tank(a, Vec2::new(100.0, 100.0));
        simulation.state.add_tank(b, Vec2::new(180.0, 100.0));
        simulation.state.tanks[0].rotation_radians = 0.0;
        simulation.state.fire(a, &simulation.config).unwrap();
        for _ in 0..30 {
            simulation.step(&[]);
        }
        assert!(!simulation.state.tanks[1].alive);
    }
}
