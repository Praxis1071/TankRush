use crate::game::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub radius: f32,
    pub kind: ParticleKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    Spark,
    Smoke,
    Muzzle,
    Explosion,
}

#[derive(Debug, Default, Clone)]
pub struct VfxSystem {
    pub particles: Vec<Particle>,
    accumulator: f32,
    seed: u32,
}

impl VfxSystem {
    pub fn new(seed: u32) -> Self {
        Self { seed, ..Self::default() }
    }

    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed as f32) / (u32::MAX as f32)
    }

    pub fn update(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.position = particle.position + particle.velocity * dt;
            particle.velocity = particle.velocity * (1.0 - (dt * 2.5).min(0.9));
            particle.life -= dt;
        }
        self.particles.retain(|particle| particle.life > 0.0);
    }

    pub fn trail(&mut self, position: Vec2, velocity: Vec2) {
        if self.particles.len() > 900 {
            return;
        }
        let direction = velocity.normalized();
        self.particles.push(Particle {
            position: position - direction * 3.0,
            velocity: -direction * 8.0,
            life: 0.16,
            max_life: 0.16,
            radius: 2.0,
            kind: ParticleKind::Smoke,
        });
    }

    pub fn muzzle(&mut self, position: Vec2, direction: Vec2) {
        for _ in 0..4 {
            let spread = (self.random() - 0.5) * 0.45;
            let angle = direction.y.atan2(direction.x) + spread;
            self.particles.push(Particle {
                position,
                velocity: Vec2::new(angle.cos(), angle.sin()) * (30.0 + self.random() * 35.0),
                life: 0.12,
                max_life: 0.12,
                radius: 2.5,
                kind: ParticleKind::Muzzle,
            });
        }
    }

    pub fn ricochet(&mut self, position: Vec2, normal: Vec2) {
        for _ in 0..7 {
            let tangent = Vec2::new(-normal.y, normal.x);
            let spread = (self.random() - 0.5) * 1.4;
            let direction = (normal + tangent * spread).normalized();
            self.particles.push(Particle {
                position,
                velocity: direction * (35.0 + self.random() * 60.0),
                life: 0.24,
                max_life: 0.24,
                radius: 1.7,
                kind: ParticleKind::Spark,
            });
        }
    }

    pub fn explosion(&mut self, position: Vec2) {
        for _ in 0..28 {
            let angle = self.random() * std::f32::consts::TAU;
            let speed = 25.0 + self.random() * 110.0;
            self.particles.push(Particle {
                position,
                velocity: Vec2::new(angle.cos(), angle.sin()) * speed,
                life: 0.35 + self.random() * 0.45,
                max_life: 0.8,
                radius: 2.0 + self.random() * 3.0,
                kind: ParticleKind::Explosion,
            });
        }
    }

    pub fn ambient(&mut self, dt: f32, position: Vec2) {
        self.accumulator += dt;
        if self.accumulator < 0.16 {
            return;
        }
        self.accumulator = 0.0;
        if self.particles.len() < 700 {
            self.particles.push(Particle {
                position,
                velocity: Vec2::new((self.random() - 0.5) * 5.0, -6.0 - self.random() * 5.0),
                life: 0.45,
                max_life: 0.45,
                radius: 1.3,
                kind: ParticleKind::Smoke,
            });
        }
    }
}
