use std::fs;
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
        Self {
            audio: AudioSettings::default(),
            map_size: "Medium Arena".into(),
            player_count: 2,
        }
    }
}

impl UserSettings {
    fn path() -> PathBuf {
        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(config_dir).join("tankrush").join("settings.conf");
        }
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("tankrush")
            .join("settings.conf")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "music" => settings.audio.music_enabled = value.trim() == "true",
                "effects" => settings.audio.sound_effects_enabled = value.trim() == "true",
                "map_size" => settings.map_size = value.trim().to_owned(),
                "player_count" => {
                    settings.player_count = value.trim().parse().unwrap_or(2).clamp(1, 4);
                }
                _ => {}
            }
        }
        settings
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let text = format!(
            "music={}\neffects={}\nmap_size={}\nplayer_count={}\n",
            self.audio.music_enabled,
            self.audio.sound_effects_enabled,
            self.map_size,
            self.player_count
        );
        let _ = fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let settings = UserSettings::default();
        assert_eq!(settings.player_count, 2);
        assert!(settings.audio.music_enabled);
    }
}
