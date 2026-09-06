use super::{collision, GameConfig, GameMap, GameState, PlayerId, TankInput, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct AiController {
    pub target: Option<PlayerId>,
    pub fire_cooldown: f32,
    pub think_timer: f32,
    pub path_timer: f32,
    pub strafe_timer: f32,
    pub strafe_sign: f32,
}

impl Default for AiController {
    fn default() -> Self {
        Self {
            target: None,
            fire_cooldown: 0.0,
            think_timer: 0.0,
            path_timer: 0.0,
            strafe_timer: 0.0,
            strafe_sign: 1.0,
        }
    }
}

impl AiController {
    pub fn input(
        &mut self,
        state: &GameState,
        map: &GameMap,
        config: &GameConfig,
        self_id: PlayerId,
        dt: f32,
    ) -> TankInput {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        self.think_timer -= dt;
        self.path_timer -= dt;
        self.strafe_timer -= dt;
        if self.think_timer <= 0.0 {
            self.target = best_target(state, map, self_id);
            self.think_timer = 0.08;
        }
        if self.strafe_timer <= 0.0 {
            self.strafe_sign = -self.strafe_sign;
            self.strafe_timer = 0.9;
        }

        let Some(tank) = state
            .tanks
            .iter()
            .find(|tank| tank.player_id == self_id && tank.alive)
        else {
            return TankInput::idle();
        };
        let Some(target_id) = self.target else {
            return TankInput::idle();
        };
        let Some(target) = state
            .tanks
            .iter()
            .find(|tank| tank.player_id == target_id && tank.alive)
        else {
            return TankInput::idle();
        };

        let to_target = target.position - tank.position;
        let distance = to_target.length();
        let direct_angle = to_target.y.atan2(to_target.x);
        let mut desired_angle = direct_angle;

        if let Some(waypoint) = next_waypoint(tank.position, target.position, map, config.tank_radius)
        {
            let to_waypoint = waypoint - tank.position;
            if to_waypoint.length() > 24.0 {
                desired_angle = to_waypoint.y.atan2(to_waypoint.x);
            }
        }

        let threat = projectile_threat(state, tank.position, config.tank_radius);
        if threat > 0.0 {
            desired_angle = threat_escape_angle(state, tank.position, desired_angle, threat);
        } else if distance < 260.0 && line_of_sight(tank.position, target.position, map) {
            desired_angle = direct_angle;
            if distance < 190.0 {
                desired_angle += 0.22 * self.strafe_sign;
            }
        }

        let delta = angle_delta(desired_angle, tank.rotation_radians);
        let aligned = angle_delta(direct_angle, tank.rotation_radians).abs() < 0.10;
        let clear_shot = ricochet_can_hit(tank.position, tank.direction(), target.position, map);
        let fire = aligned && (clear_shot || distance < 120.0) && self.fire_cooldown <= 0.0;

        if fire {
            self.fire_cooldown = if clear_shot { 0.34 } else { 0.48 };
        }

        let obstacle_ahead = wall_ahead(tank.position, tank.direction(), map, 42.0);
        let reverse = distance < 72.0 && aligned && !threat.is_sign_positive();
        TankInput {
            forward: !reverse && (!obstacle_ahead || delta.abs() > 0.45),
            backward: reverse,
            left: delta < -0.055,
            right: delta > 0.055,
            fire,
        }
    }
}

fn angle_delta(target: f32, current: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn best_target(state: &GameState, map: &GameMap, self_id: PlayerId) -> Option<PlayerId> {
    let origin = state
        .tanks
        .iter()
        .find(|tank| tank.player_id == self_id && tank.alive)?
        .position;
    state
        .tanks
        .iter()
        .filter(|tank| tank.alive && tank.player_id != self_id)
        .map(|tank| {
            let distance = (tank.position - origin).length();
            let visibility = if line_of_sight(origin, tank.position, map) {
                -90.0
            } else {
                0.0
            };
            (distance + visibility, tank.player_id)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

fn line_of_sight(start: Vec2, end: Vec2, map: &GameMap) -> bool {
    !map.walls
        .iter()
        .any(|wall| collision::segment_wall_hit(start, end, *wall).is_some())
}

fn next_waypoint(position: Vec2, target: Vec2, map: &GameMap, radius: f32) -> Option<Vec2> {
    let (cells_x, cells_y) = map.size.dimensions();
    let rooms_x = ((cells_x - 2) / 2) as i32;
    let rooms_y = ((cells_y - 2) / 2) as i32;
    let room_size = 64.0;
    let center = |x: i32, y: i32| Vec2::new(48.0 + x as f32 * room_size, 48.0 + y as f32 * room_size);
    let nearest = |point: Vec2| {
        let x = ((point.x - 48.0) / room_size).round().clamp(0.0, (rooms_x - 1) as f32) as i32;
        let y = ((point.y - 48.0) / room_size).round().clamp(0.0, (rooms_y - 1) as f32) as i32;
        (x, y)
    };
    let start = nearest(position);
    let goal = nearest(target);
    if start == goal {
        return None;
    }

    let mut queue = std::collections::VecDeque::from([start]);
    let mut previous = vec![None::<(i32, i32)>; (rooms_x * rooms_y) as usize];
    let mut visited = vec![false; (rooms_x * rooms_y) as usize];
    let index = |x: i32, y: i32| (y * rooms_x + x) as usize;
    visited[index(start.0, start.1)] = true;
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == goal {
            break;
        }
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if nx < 0 || ny < 0 || nx >= rooms_x || ny >= rooms_y || visited[index(nx, ny)] {
                continue;
            }
            let a = center(x, y);
            let b = center(nx, ny);
            if map
                .walls
                .iter()
                .any(|wall| collision::segment_wall_hit(a, b, *wall).is_some())
            {
                continue;
            }
            visited[index(nx, ny)] = true;
            previous[index(nx, ny)] = Some((x, y));
            queue.push_back((nx, ny));
        }
    }
    if !visited[index(goal.0, goal.1)] {
        return None;
    }
    let mut current = goal;
    while previous[index(current.0, current.1)] != Some(start) {
        current = previous[index(current.0, current.1)]?;
    }
    let waypoint = center(current.0, current.1);
    if map.walls.iter().any(|wall| collision::circle_intersects_wall(waypoint, radius, *wall)) {
        None
    } else {
        Some(waypoint)
    }
}

fn wall_ahead(position: Vec2, direction: Vec2, map: &GameMap, distance: f32) -> bool {
    let end = position + direction.normalized() * distance;
    map.walls
        .iter()
        .any(|wall| collision::segment_wall_hit(position, end, *wall).is_some())
}

fn projectile_threat(state: &GameState, position: Vec2, radius: f32) -> f32 {
    state
        .projectiles
        .iter()
        .filter_map(|projectile| {
            let velocity = projectile.velocity.normalized();
            let to_tank = position - projectile.position;
            if velocity.dot(to_tank) <= 0.0 {
                return None;
            }
            let projection = to_tank.dot(velocity);
            let closest = projectile.position + velocity * projection;
            let miss = (position - closest).length();
            if miss <= radius + 26.0 && projection < 180.0 {
                Some((180.0 - projection).max(1.0) / (miss + 8.0))
            } else {
                None
            }
        })
        .fold(0.0, f32::max)
}

fn threat_escape_angle(state: &GameState, position: Vec2, base: f32, threat: f32) -> f32 {
    let mut escape = Vec2::ZERO;
    for projectile in &state.projectiles {
        let to_tank = position - projectile.position;
        if projectile.velocity.normalized().dot(to_tank) > 0.0 && to_tank.length() < 220.0 {
            let velocity = projectile.velocity.normalized();
            let side = Vec2::new(-velocity.y, velocity.x);
            let sign = if side.dot(to_tank) >= 0.0 { 1.0 } else { -1.0 };
            escape = escape + side * sign * threat;
        }
    }
    if escape.length() <= f32::EPSILON {
        base
    } else {
        let desired = escape.normalized();
        desired.y.atan2(desired.x)
    }
}

fn ricochet_can_hit(origin: Vec2, direction: Vec2, target: Vec2, map: &GameMap) -> bool {
    let mut position = origin;
    let mut velocity = direction.normalized();
    for _ in 0..3 {
        let end = position + velocity * 520.0;
        let mut nearest = None;
        for wall in &map.walls {
            if let Some(hit) = collision::segment_wall_hit(position, end, *wall) {
                if nearest.is_none_or(|current: collision::CollisionHit| hit.distance < current.distance) {
                    nearest = Some(hit);
                }
            }
        }
        if (target - position).length() <= 520.0 {
            let to_target = target - position;
            if velocity.dot(to_target.normalized()) > 0.995 {
                return true;
            }
        }
        let Some(hit) = nearest else {
            return false;
        };
        position = hit.point + hit.normal * 0.5;
        velocity = velocity.reflect(hit.normal).normalized();
    }
    false
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
        let input = controller.input(
            &sim.state,
            &sim.map,
            &sim.config,
            ai,
            1.0 / 60.0,
        );
        assert!(input.right || input.forward || input.fire);
        assert_eq!(controller.target, Some(enemy));
    }

    #[test]
    fn ai_can_find_a_connected_waypoint() {
        let map = GameMap::generate(MapSize::Medium);
        let waypoint = next_waypoint(Vec2::new(48.0, 48.0), Vec2::new(240.0, 240.0), &map, 14.0);
        assert!(waypoint.is_some());
    }
}
