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

    const fn seed(self) -> u32 {
        match self {
            Self::Small => 0x1357_9BDF,
            Self::Medium => 0x2468_ACE1,
            Self::Large => 0x5A17_C0DE,
            Self::VeryLarge => 0x71A9_42EF,
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
        Self::generate(size)
    }

    pub fn generate(size: MapSize) -> Self {
        Self::generate_seeded(size, size.seed())
    }

    pub fn generate_seeded(size: MapSize, initial_seed: u32) -> Self {
        let (cells_x_total, cells_y_total) = size.dimensions();
        let cell = 32.0;
        let width = cells_x_total as f32 * cell;
        let height = cells_y_total as f32 * cell;
        let thickness = 14.0;
        let cells_x = (cells_x_total - 2) as usize;
        let cells_y = (cells_y_total - 2) as usize;
        let rooms_x = cells_x / 2;
        let rooms_y = cells_y / 2;
        let room_count = rooms_x * rooms_y;

        let mut walls = vec![
            Wall::new(Vec2::new(0.0, 0.0), Vec2::new(width, thickness)),
            Wall::new(Vec2::new(0.0, height - thickness), Vec2::new(width, height)),
            Wall::new(Vec2::new(0.0, 0.0), Vec2::new(thickness, height)),
            Wall::new(Vec2::new(width - thickness, 0.0), Vec2::new(width, height)),
        ];

        // Build a connected room graph first, then open most of its remaining
        // edges. This intentionally avoids the dense one-cell maze look: the
        // player gets broad routes, cover and bank-shot angles instead.
        let mut visited = vec![false; room_count];
        let mut passages = vec![[false; 4]; room_count];
        let mut stack = vec![(0usize, 0usize)];
        visited[0] = true;
        let mut seed = initial_seed.max(1);

        while let Some(&(x, y)) = stack.last() {
            let mut options = [(0usize, 0usize, 0usize, 0usize); 4];
            let mut option_count = 0;
            let candidates = [
                (0isize, -1isize, 0usize, 2usize),
                (1, 0, 1, 3),
                (0, 1, 2, 0),
                (-1, 0, 3, 1),
            ];
            for &(dx, dy, direction, opposite) in &candidates {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < rooms_x
                    && (ny as usize) < rooms_y
                    && !visited[ny as usize * rooms_x + nx as usize]
                {
                    options[option_count] = (nx as usize, ny as usize, direction, opposite);
                    option_count += 1;
                }
            }
            if option_count == 0 {
                stack.pop();
                continue;
            }
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let choice = (seed as usize) % option_count;
            let (nx, ny, direction, opposite) = options[choice];
            let current = y * rooms_x + x;
            let next = ny * rooms_x + nx;
            passages[current][direction] = true;
            passages[next][opposite] = true;
            visited[next] = true;
            stack.push((nx, ny));
        }

        // Open about 75% of the still-closed adjacencies. The spanning tree
        // guarantees connectivity; the extra openings make the arena sparse.
        let extra_openings = (room_count * 3 / 4).max(1);
        for _ in 0..extra_openings {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let x = (seed as usize) % rooms_x;
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let y = (seed as usize) % rooms_y;
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let direction = (seed as usize) % 4;
            let (dx, dy, opposite) = match direction {
                0 => (0isize, -1isize, 2usize),
                1 => (1, 0, 3),
                2 => (0, 1, 0),
                _ => (-1, 0, 1),
            };
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < rooms_x && (ny as usize) < rooms_y {
                let current = y * rooms_x + x;
                let next = ny as usize * rooms_x + nx as usize;
                passages[current][direction] = true;
                passages[next][opposite] = true;
            }
        }

        for y in 0..rooms_y {
            for x in 0..rooms_x {
                let index = y * rooms_x + x;
                let left = 16.0 + x as f32 * cell * 2.0;
                let top = 16.0 + y as f32 * cell * 2.0;
                if x + 1 < rooms_x && !passages[index][1] {
                    let wall_x = left + cell * 2.0;
                    walls.push(Wall::new(
                        Vec2::new(wall_x - thickness / 2.0, top),
                        Vec2::new(wall_x + thickness / 2.0, top + cell * 2.0),
                    ));
                }
                if y + 1 < rooms_y && !passages[index][2] {
                    let wall_y = top + cell * 2.0;
                    walls.push(Wall::new(
                        Vec2::new(left, wall_y - thickness / 2.0),
                        Vec2::new(left + cell * 2.0, wall_y + thickness / 2.0),
                    ));
                }
            }
        }

        let mut candidates = vec![
            (0usize, 0usize),
            (rooms_x - 1, 0),
            (0, rooms_y - 1),
            (rooms_x - 1, rooms_y - 1),
            (rooms_x / 2, 0),
            (rooms_x / 2, rooms_y - 1),
            (0, rooms_y / 2),
            (rooms_x - 1, rooms_y / 2),
            (rooms_x / 3, rooms_y / 3),
            ((rooms_x * 2) / 3, (rooms_y * 2) / 3),
        ];
        // Shuffle spawn order so every seeded arena has a different opening.
        for index in (1..candidates.len()).rev() {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let swap = (seed as usize) % (index + 1);
            candidates.swap(index, swap);
        }
        let spawn_points = candidates
            .into_iter()
            .map(|(x, y)| {
                Vec2::new(
                    16.0 + x as f32 * cell * 2.0 + cell,
                    16.0 + y as f32 * cell * 2.0 + cell,
                )
            })
            .collect();

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
    fn generated_map_is_sparse_enough_for_open_combat() {
        let map = GameMap::generate(MapSize::Medium);
        assert!(map.walls.len() > 4);
        assert!(map.walls.len() < 60);
    }

    #[test]
    fn generated_map_has_ten_safe_spawns() {
        let map = GameMap::generate(MapSize::Medium);
        assert_eq!(map.spawn_points().len(), 10);
        assert!(map.spawn_points().iter().all(|&p| {
            map.is_inside_play_area(p, 14.0)
                && !map.walls.iter().any(|wall| wall.contains(p))
        }));
    }

    #[test]
    fn generated_map_is_deterministic() {
        let first = GameMap::generate(MapSize::Large);
        let second = GameMap::generate(MapSize::Large);
        assert_eq!(first.walls, second.walls);
        assert_eq!(first.spawn_points, second.spawn_points);
    }

    #[test]
    fn different_seeds_change_the_arena() {
        let first = GameMap::generate_seeded(MapSize::Medium, 1);
        let second = GameMap::generate_seeded(MapSize::Medium, 2);
        assert_ne!(first.walls, second.walls);
        assert_ne!(first.spawn_points, second.spawn_points);
    }
}
