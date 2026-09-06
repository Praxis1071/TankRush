use super::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSize {
    Small,
    Medium,
    Large,
    VeryLarge,
}

impl MapSize {
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Small => (24, 18),
            Self::Medium => (32, 24),
            Self::Large => (42, 30),
            Self::VeryLarge => (56, 40),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wall {
    pub min: Vec2,
    pub max: Vec2,
}

impl Wall {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

#[derive(Debug, Clone)]
pub struct GameMap {
    pub size: MapSize,
    pub width: f32,
    pub height: f32,
    pub walls: Vec<Wall>,
    pub spawn_points: Vec<Vec2>,
}

impl GameMap {
    pub fn rectangular(size: MapSize) -> Self {
        let (cells_x, cells_y) = size.dimensions();
        let width = cells_x as f32 * 32.0;
        let height = cells_y as f32 * 32.0;
        let thickness = 16.0;
        let walls = vec![
            Wall::new(Vec2::new(0.0, 0.0), Vec2::new(width, thickness)),
            Wall::new(
                Vec2::new(0.0, height - thickness),
                Vec2::new(width, height),
            ),
            Wall::new(Vec2::new(0.0, 0.0), Vec2::new(thickness, height)),
            Wall::new(
                Vec2::new(width - thickness, 0.0),
                Vec2::new(width, height),
            ),
        ];
        let spawn_points = vec![
            Vec2::new(width * 0.2, height * 0.2),
            Vec2::new(width * 0.8, height * 0.2),
            Vec2::new(width * 0.2, height * 0.8),
            Vec2::new(width * 0.8, height * 0.8),
        ];
        Self {
            size,
            width,
            height,
            walls,
            spawn_points,
        }
    }

    pub fn is_inside_play_area(&self, point: Vec2, radius: f32) -> bool {
        point.x - radius >= 16.0
            && point.y - radius >= 16.0
            && point.x + radius <= self.width - 16.0
            && point.y + radius <= self.height - 16.0
    }

    pub fn spawn_points(&self) -> &[Vec2] {
        &self.spawn_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_sizes_grow_monotonically() {
        let small = MapSize::Small.dimensions();
        let large = MapSize::VeryLarge.dimensions();
        assert!(large.0 > small.0 && large.1 > small.1);
    }

    #[test]
    fn rectangular_map_has_safe_spawns() {
        let map = GameMap::rectangular(MapSize::Medium);
        assert!(map
            .spawn_points()
            .iter()
            .all(|&p| map.is_inside_play_area(p, 14.0)));
    }
}
