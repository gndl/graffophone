use gpplugin::audio_format::AudioFormat;
use gpplugin::ear;
use gpplugin::talker::{Talker, TalkerBase};
use gpplugin::talker_handler::TalkerHandlerBase;
use gpplugin::voice;
use std::f64::consts::PI;

pub struct Sinusoidal {
    base: TalkerBase,
    last_tick: i64,
    last_freq: f64,
    last_angle: f64,
}

impl Sinusoidal {
    pub fn new() -> Sinusoidal {
        let mut base = TalkerBase::new();

        let freq = ear::audio(Some("frequence".to_string()), Some(440.), None);
        base.add_ear(freq);

        let phase = ear::audio(Some("phase".to_string()), Some(0.), None);
        base.add_ear(phase);

        let voice = voice::audio(None, None, None);
        base.add_voice(voice);

        Self {
            base,
            last_tick: 0,
            last_freq: 0.,
            last_angle: 0.,
        }
    }

    pub fn id() -> &'static str {
        "Sinusoidal"
    }
    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new(Sinusoidal::id(), "Sinusoidal", "Generator")
    }
}

impl Talker for Sinusoidal {
    fn base<'a>(&'a self) -> &'a TalkerBase {
        &self.base
    }

    fn talk(&mut self, _port: usize, tick: i64, len: usize) -> usize {
        let mut ln = len;
        let c = AudioFormat::frequence_coef();
        let mut last_freq = 0.;
        let mut last_angle = 0.;

        if self.last_tick == tick {
            last_freq = self.last_freq;
            last_angle = self.last_angle;
        }

        for ear in self.ears() {
            ln = ear.listen(tick, ln);
        }
        for voice in self.voices() {
            let freq_buf = self.ear_audio_buffer(0).unwrap();
            let phase_buf = self.ear_audio_buffer(1).unwrap();

            let mut vc = voice.borrow_mut();
            let voice_buf = vc.audio_buffer().unwrap();

            for i in 0..ln {
                let p = phase_buf[i].get() as f64 * PI;
                let a = last_angle + last_freq * c;

                let sample = (a + p).sin() as f32;
                voice_buf.get()[i].set(sample);
                last_freq = freq_buf[i].get() as f64;
                last_angle = a;
            }
            vc.set_len(ln);
            vc.set_tick(tick);
        }
        self.last_freq = last_freq;
        self.last_angle = last_angle;
        self.last_tick = tick + ln as i64;
        ln
    }
}
