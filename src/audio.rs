use std::time::Duration;

use rodio::{DeviceSinkBuilder, Player, Source};

#[derive(Debug, Clone, Copy)]
pub enum SoundEvent {
    Fire,
    Ricochet,
    Pickup,
    Explosion,
    RoundWin,
}

pub struct AudioEngine {
    sink: Option<rodio::MixerDeviceSink>,
    pub enabled: bool,
}

impl AudioEngine {
    pub fn new(enabled: bool) -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok();
        Self { sink, enabled }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn play(&self, event: SoundEvent) {
        if !self.enabled {
            return;
        }
        let Some(sink) = &self.sink else {
            return;
        };
        let (frequency, duration, volume) = match event {
            SoundEvent::Fire => (190.0, 0.07, 0.18),
            SoundEvent::Ricochet => (820.0, 0.045, 0.10),
            SoundEvent::Pickup => (640.0, 0.11, 0.12),
            SoundEvent::Explosion => (90.0, 0.22, 0.20),
            SoundEvent::RoundWin => (520.0, 0.32, 0.15),
        };
        let player = Player::connect_new(sink.mixer());
        let tone = rodio::source::SineWave::new(frequency)
            .take_duration(Duration::from_secs_f32(duration))
            .amplify(volume)
            .fade_in(Duration::from_millis(4))
            .fade_out(Duration::from_millis(18));
        player.append(tone);
    }
}
