use super::{Vec2, map::Wall};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionHit {
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

pub fn circle_intersects_wall(center: Vec2, radius: f32, wall: Wall) -> bool {
    let closest = Vec2::new(
        center.x.clamp(wall.min.x, wall.max.x),
        center.y.clamp(wall.min.y, wall.max.y),
    );
    let delta = center - closest;
    delta.dot(delta) <= radius * radius
}

pub fn reflect_velocity(velocity: Vec2, normal: Vec2) -> Vec2 {
    velocity.reflect(normal.normalized())
}

pub fn segment_wall_hit(start: Vec2, end: Vec2, wall: Wall) -> Option<CollisionHit> {
    let delta = end - start;
    let mut best: Option<CollisionHit> = None;

    for (axis, min, max) in [
        (0usize, wall.min.x, wall.max.x),
        (1usize, wall.min.y, wall.max.y),
    ] {
        let component = if axis == 0 { delta.x } else { delta.y };
        if component.abs() <= f32::EPSILON {
            continue;
        }
        for boundary in [min, max] {
            let start_component = if axis == 0 { start.x } else { start.y };
            let t = (boundary - start_component) / component;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let point = start + delta * t;
            let inside_other_axis = if axis == 0 {
                point.y >= wall.min.y && point.y <= wall.max.y
            } else {
                point.x >= wall.min.x && point.x <= wall.max.x
            };
            if !inside_other_axis {
                continue;
            }
            let normal = match (axis, boundary == min) {
                (0, true) => Vec2::new(-1.0, 0.0),
                (0, false) => Vec2::new(1.0, 0.0),
                (1, true) => Vec2::new(0.0, -1.0),
                _ => Vec2::new(0.0, 1.0),
            };
            let hit = CollisionHit {
                point,
                normal,
                distance: delta.length() * t,
            };
            if best.is_none_or(|current| hit.distance < current.distance) {
                best = Some(hit);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_wall_collision_detects_overlap() {
        let wall = Wall::new(Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0));
        assert!(circle_intersects_wall(Vec2::new(10.0, 15.0), 1.0, wall));
        assert!(!circle_intersects_wall(Vec2::new(0.0, 0.0), 1.0, wall));
    }

    #[test]
    fn reflection_uses_surface_normal() {
        let result = reflect_velocity(Vec2::new(10.0, -5.0), Vec2::new(0.0, 1.0));
        assert_eq!(result, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn segment_hit_finds_wall_face() {
        let wall = Wall::new(Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0));
        let hit =
            segment_wall_hit(Vec2::new(0.0, 15.0), Vec2::new(15.0, 15.0), wall).unwrap();
        assert!((hit.point.x - 10.0).abs() < 0.001);
        assert_eq!(hit.normal, Vec2::new(-1.0, 0.0));
    }
}
