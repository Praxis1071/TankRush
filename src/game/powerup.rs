use super::{GameMap, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    DoubleShot,
    MachineGun,
    Laser,
    GuidedMissile,
    Shrapnel,
    Mine,
}

impl PowerUpKind {
    pub const ALL: [Self; 6] = [
        Self::DoubleShot,
        Self::MachineGun,
        Self::Laser,
        Self::GuidedMissile,
        Self::Shrapnel,
        Self::Mine,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::DoubleShot => "DOUBLE",
            Self::MachineGun => "RAPID",
            Self::Laser => "LASER",
            Self::GuidedMissile => "GUIDED",
            Self::Shrapnel => "FRAG",
            Self::Mine => "MINE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerUp {
    pub position: Vec2,
    pub kind: PowerUpKind,
}

impl PowerUp {
    pub fn generate(map: &GameMap, seed: &mut u32) -> Self {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let index = (*seed as usize) % map.spawn_points().len();
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let kind = PowerUpKind::ALL[(*seed as usize) % PowerUpKind::ALL.len()];
        Self {
            position: map.spawn_points()[index],
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MapSize;

    #[test]
    fn seeded_powerup_generation_is_repeatable() {
        let map = GameMap::generate(MapSize::Medium);
        let mut a = 7;
        let mut b = 7;
        assert_eq!(
            PowerUp::generate(&map, &mut a),
            PowerUp::generate(&map, &mut b)
        );
    }

    #[test]
    fn all_weapon_kinds_have_stable_labels() {
        assert_eq!(PowerUpKind::ALL.len(), 6);
        assert_eq!(PowerUpKind::Mine.label(), "MINE");
    }
}
