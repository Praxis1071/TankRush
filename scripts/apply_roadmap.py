from pathlib import Path


def write(path: str, content: str) -> None:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content)


cargo = Path("Cargo.toml")
cargo_text = cargo.read_text()
if "rodio = " not in cargo_text:
    cargo.write_text(cargo_text.replace(
        'glib = "0.21"',
        'glib = "0.21"\nrodio = { version = "0.22", default-features = false, features = ["playback"] }',
    ))

write("src/settings.rs", '''use std::fs;
use std::path::PathBuf;

use crate::game::config::AudioSettings;

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub audio: AudioSettings,
    pub map_size: String,
    pub player_count: usize,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self { audio: AudioSettings::default(), map_size: "Medium Arena".into(), player_count: 2 }
    }
}

impl UserSettings {
    fn path() -> PathBuf {
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(dir).join("tankrush/settings.conf");
        }
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
            .join(".config/tankrush/settings.conf")
    }

    pub fn load() -> Self {
        let Ok(text) = fs::read_to_string(Self::path()) else { return Self::default(); };
        let mut out = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else { continue; };
            match key.trim() {
                "music" => out.audio.music_enabled = value.trim() == "true",
                "effects" => out.audio.sound_effects_enabled = value.trim() == "true",
                "map_size" => out.map_size = value.trim().to_owned(),
                "player_count" => out.player_count = value.trim().parse().unwrap_or(2).clamp(1, 4),
                _ => {}
            }
        }
        out
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
        let text = format!("music={}\\neffects={}\\nmap_size={}\\nplayer_count={}\\n",
            self.audio.music_enabled, self.audio.sound_effects_enabled, self.map_size, self.player_count);
        let _ = fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_sane() { assert_eq!(UserSettings::default().player_count, 2); }
}
''')

write("src/vfx.rs", '''use crate::game::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub radius: f32,
}

#[derive(Debug, Default, Clone)]
pub struct VfxSystem { pub particles: Vec<Particle>, seed: u32 }

impl VfxSystem {
    pub fn new(seed: u32) -> Self { Self { seed, ..Self::default() } }
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.seed as f32 / u32::MAX as f32
    }
    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position = p.position + p.velocity * dt;
            p.velocity = p.velocity * (1.0 - (dt * 2.5).min(0.9));
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }
    pub fn muzzle(&mut self, position: Vec2, direction: Vec2) {
        for _ in 0..4 {
            let angle = direction.y.atan2(direction.x) + (self.random() - 0.5) * 0.45;
            self.particles.push(Particle {
                position,
                velocity: Vec2::new(angle.cos(), angle.sin()) * (30.0 + self.random() * 35.0),
                life: 0.12, max_life: 0.12, radius: 2.5,
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
                life: 0.35 + self.random() * 0.45, max_life: 0.8,
                radius: 2.0 + self.random() * 3.0,
            });
        }
    }
}
''')

write("src/audio.rs", '''use std::time::Duration;
use rodio::{DeviceSinkBuilder, Source};

#[derive(Debug, Clone, Copy)]
pub enum SoundEvent { Fire, Ricochet, Pickup, Explosion, RoundWin }

pub struct AudioEngine {
    sink: Option<rodio::MixerDeviceSink>,
    pub enabled: bool,
}

impl AudioEngine {
    pub fn new(enabled: bool) -> Self {
        Self { sink: DeviceSinkBuilder::open_default_sink().ok(), enabled }
    }
    pub fn play(&self, event: SoundEvent) {
        if !self.enabled { return; }
        let Some(sink) = &self.sink else { return; };
        let (freq, secs, vol) = match event {
            SoundEvent::Fire => (190.0, 0.07, 0.18),
            SoundEvent::Ricochet => (820.0, 0.045, 0.10),
            SoundEvent::Pickup => (640.0, 0.11, 0.12),
            SoundEvent::Explosion => (90.0, 0.22, 0.20),
            SoundEvent::RoundWin => (520.0, 0.32, 0.15),
        };
        sink.mixer().add(rodio::source::SineWave::new(freq)
            .take_duration(Duration::from_secs_f32(secs))
            .amplify_normalized(vol)
            .fade_in(Duration::from_millis(4))
            .fade_out(Duration::from_millis(18)));
    }
}
''')

main = Path("src/main.rs")
s = main.read_text()
if "mod audio;" not in s:
    s = "mod audio;\n" + s
if "mod settings;" not in s:
    s = s.replace("mod game;\n", "mod game;\nmod settings;\nmod vfx;\n", 1)
if "use audio::{" not in s:
    s = s.replace(
        "use std::time::{Duration, SystemTime, UNIX_EPOCH};",
        "use std::time::{Duration, SystemTime, UNIX_EPOCH};\nuse audio::{AudioEngine, SoundEvent};\nuse settings::UserSettings;\nuse vfx::VfxSystem;",
    )
s = s.replace(
    "    paused: bool,\n}",
    "    paused: bool,\n    guided_projectiles: Vec<game::ProjectileId>,\n    vfx: VfxSystem,\n    audio: AudioEngine,\n}", 1)
s = s.replace(
    "            paused: false,\n        };",
    "            paused: false,\n            guided_projectiles: Vec::new(),\n            vfx: VfxSystem::new(seed ^ 0xC0FF_EE11),\n            audio: AudioEngine::new(true),\n        };", 1)
s = s.replace(
    "        self.paused = false;\n        let spawns",
    "        self.paused = false;\n        self.guided_projectiles.clear();\n        self.vfx.particles.clear();\n        let spawns", 1)
s = s.replace(
    "                    &self.simulation.config,\n                    ai_id,\n                    dt,",
    "                    &self.simulation.config,\n                    &self.powerups,\n                    ai_id,\n                    dt,", 1)
s = s.replace(
    "                    projectile.velocity = projectile.velocity * 0.92;\n                    projectile.remaining_lifetime = 5.0;",
    "                    projectile.velocity = projectile.velocity * 0.92;\n                    projectile.remaining_lifetime = 5.0;\n                    self.guided_projectiles.push(projectile.id);", 1)
start = s.find("    fn guide_projectiles(&mut self) {")
end = s.find("\n    fn toggle_pause", start)
if start >= 0 and end > start:
    s = s[:start] + '''    fn guide_projectiles(&mut self) {
        self.guided_projectiles.retain(|id| self.simulation.state.projectiles.iter().any(|p| p.id == *id));
        let targets = self.simulation.state.tanks.iter().filter(|t| t.alive)
            .map(|t| (t.player_id, t.position)).collect::<Vec<_>>();
        for projectile in &mut self.simulation.state.projectiles {
            if !self.guided_projectiles.contains(&projectile.id) { continue; }
            let Some((_, target)) = targets.iter().filter(|(id, _)| *id != projectile.owner)
                .min_by(|a, b| ((a.1 - projectile.position).length()).total_cmp(&((b.1 - projectile.position).length()))) else { continue; };
            let desired = (*target - projectile.position).normalized();
            let speed = projectile.velocity.length();
            projectile.velocity = (projectile.velocity.normalized() * 0.84 + desired * 0.16).normalized() * speed;
        }
    }
''' + s[end:]
main.write_text(s)

# AI receives the power-up field; this keeps Laika aware of objectives without coupling it to GTK.
ai = Path("src/game/ai.rs")
a = ai.read_text()
a = a.replace("use super::{collision, GameConfig, GameMap, GameState, PlayerId, TankInput, Vec2};",
              "use super::{collision, GameConfig, GameMap, GameState, PlayerId, PowerUp, TankInput, Vec2};")
a = a.replace("        config: &GameConfig,\n        self_id: PlayerId,",
              "        config: &GameConfig,\n        powerups: &[PowerUp],\n        self_id: PlayerId,", 1)
a = a.replace("controller.input(&sim.state, &sim.map, &sim.config, ai, 1.0 / 60.0)",
              "controller.input(&sim.state, &sim.map, &sim.config, &[], ai, 1.0 / 60.0)")
ai.write_text(a)

# Prevent tanks from occupying the same space.
sim = Path("src/game/simulation.rs")
x = sim.read_text()
needle = """            let tank = &mut self.state.tanks[index];
            tank.rotation_radians = rotation;
            tank.position = new_position;"""
if needle in x:
    x = x.replace(needle, """            let mut resolved_position = new_position;
            for (other_index, other) in self.state.tanks.iter().enumerate() {
                if other_index == index || !other.alive { continue; }
                if (resolved_position - other.position).length() < self.config.tank_radius * 2.0 {
                    resolved_position = old_position;
                    break;
                }
            }
            let tank = &mut self.state.tanks[index];
            tank.rotation_radians = rotation;
            tank.position = resolved_position;""", 1)
sim.write_text(x)

# Flatpak files.
write("io.github.praxis1071.TankRush.desktop", """[Desktop Entry]
Name=TankRush
Comment=Rust GTK4 ricochet tank arena
Exec=tankrush
Icon=io.github.praxis1071.TankRush
Terminal=false
Type=Application
Categories=Game;Arcade;
""")
write("io.github.praxis1071.TankRush.metainfo.xml", """<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.github.praxis1071.TankRush</id>
  <name>TankRush</name>
  <summary>Fast local and LAN ricochet tank arena</summary>
  <project_license>MIT</project_license>
  <description><p>TankRush is a Rust and GTK4 tank arena with procedural mazes, ricochet projectiles, power-ups and local multiplayer.</p></description>
  <launchable type="desktop-id">io.github.praxis1071.TankRush.desktop</launchable>
  <url type="homepage">https://github.com/Praxis1071/TankRush</url>
</component>
""")
write("io.github.praxis1071.TankRush.yml", """id: io.github.praxis1071.TankRush
runtime: org.freedesktop.Platform
runtime-version: '25.08'
sdk: org.freedesktop.Sdk
command: tankrush
finish-args:
  - --share=ipc
  - --socket=x11
  - --socket=wayland
  - --device=dri
  - --socket=pulseaudio
  - --share=network
modules:
  - name: tankrush
    buildsystem: simple
    build-commands:
      - cargo build --release --offline
      - install -Dm755 target/release/tankrush /app/bin/tankrush
      - install -Dm644 io.github.praxis1071.TankRush.desktop /app/share/applications/io.github.praxis1071.TankRush.desktop
      - install -Dm644 io.github.praxis1071.TankRush.metainfo.xml /app/share/metainfo/io.github.praxis1071.TankRush.metainfo.xml
    sources:
      - type: dir
        path: .
""")

ci = Path(".github/workflows/ci.yml")
ci.write_text(ci.read_text().replace("libgtk-4-dev", "libgtk-4-dev libasound2-dev"))
