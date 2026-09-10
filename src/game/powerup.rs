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
            Self::DoubleShot => "◆ DOUBLE",
            Self::MachineGun => "▦ RAPID",
            Self::Laser => "ϟ LASER",
            Self::GuidedMissile => "➤ GUIDED",
            Self::Shrapnel => "✣ FRAG",
            Self::Mine => "✹ MINE",
        }
    }

    pub const fn ammo(self) -> Option<u8> {
        match self {
            Self::DoubleShot
            | Self::Laser
            | Self::GuidedMissile
            | Self::Shrapnel
            | Self::Mine => Some(1),
            Self::MachineGun => Some(8),
        }
    }

    pub const fn is_standard_augment(self) -> bool {
        matches!(self, Self::DoubleShot)
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
    fn weapon_labels_include_visual_symbols() {
        for kind in PowerUpKind::ALL {
            assert!(kind.label().chars().count() > 3);
        }
    }

    #[test]
    fn weapon_ammo_policy_is_explicit() {
        assert_eq!(PowerUpKind::DoubleShot.ammo(), Some(1));
        assert_eq!(PowerUpKind::MachineGun.ammo(), Some(8));
        assert_eq!(PowerUpKind::Laser.ammo(), Some(1));
        assert_eq!(PowerUpKind::GuidedMissile.ammo(), Some(1));
        assert_eq!(PowerUpKind::Shrapnel.ammo(), Some(1));
        assert_eq!(PowerUpKind::Mine.ammo(), Some(1));
    }

    #[test]
    fn double_shot_is_the_only_current_augment_weapon() {
        assert!(PowerUpKind::DoubleShot.is_standard_augment());
        assert!(!PowerUpKind::MachineGun.is_standard_augment());
        assert!(!PowerUpKind::Laser.is_standard_augment());
        assert!(!PowerUpKind::GuidedMissile.is_standard_augment());
        assert!(!PowerUpKind::Shrapnel.is_standard_augment());
        assert!(!PowerUpKind::Mine.is_standard_augment());
    }
}
