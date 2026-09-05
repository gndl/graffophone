use talker::audio_format::AudioFormat;
use talker::dsp;
use talker::ear::{self, Init, Ear};
use talker::identifier::Index;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talkers::parabolic::Parabolic;
use talkers::round::Round;
use talkers::sinusoidal::Sinusoidal;
use talkers::square::Square;
use talkers::triangle::Triangle;
use talker::ctalker;


pub const MODEL: &str = "Oscillator";
#[derive(PartialEq, Debug, Clone)]
enum WaveForm {
    Sin,
    Pbl,
    Tri,
    Rnd,
    Sqr,
}
impl WaveForm {
    pub fn new(selection: f32) -> WaveForm {
        if selection < 1. {
            WaveForm::Sin
        }
        else if selection < 2. {
            WaveForm::Pbl
        }
        else if selection < 3. {
            WaveForm::Tri
        }
        else if selection < 4. {
            WaveForm::Rnd
        }
        else {
            WaveForm::Sqr
        }
    }
    pub fn new_instance(&self, base: TalkerBase) -> Result<CTalker, failure::Error> {
        match self {
            WaveForm::Sin => Sinusoidal::new(base),
            WaveForm::Pbl => Parabolic::new(base),
            WaveForm::Tri => Triangle::new(base),
            WaveForm::Rnd => Round::new(base),
            WaveForm::Sqr => Square::new(base),
        }
    }
}

pub struct Oscillator {
    sample_rate: usize,
    selector_ear_index: Index,
    form: WaveForm,
    core: Box<dyn Talker>,
}

impl Oscillator {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let selector_ear_index = base.add_ear(ear::cv(Some("form"), 0., 4.99, 0., &Init::DefValue)?);

        let (base, core) = WaveForm::Sin.new_instance(base)?;
        
        Ok(ctalker!(
            base,
            Self {
                sample_rate: AudioFormat::sample_rate(),
                selector_ear_index,
                form: WaveForm::Sin,
                core,
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Oscillator", MODEL, MODEL)
    }
}

impl Talker for Oscillator {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let mut ln = base.ear(self.selector_ear_index).listen(tick, len);

        ln = self.core.talk(base, port, tick, ln);

        let selector_buf = base.ear_cv_buffer(self.selector_ear_index);

        let form = WaveForm::new(selector_buf[0]);

        if form != self.form {
            let mut b = TalkerBase::new(0, "", "", false);
            b.add_ear(Ear::new(None, false, None, None));

            let ctalker = form.new_instance(b);

            match ctalker {
                Ok((_, mut core)) => {
                    let voice_buf = base.voice(port).audio_buffer();

                    let vm2 = voice_buf[0];
                    let vm1 = voice_buf[1];

                    ln = core.talk(base, port, tick, ln);

                    dsp::recoveryless_fade_buffer(self.sample_rate, voice_buf, 2, vm2, vm1);

                    voice_buf[0] = vm2;
                    voice_buf[1] = vm1;

                    self.core = core;
                    self.form = form;
                }
                Err(_) => (),
            }
        }

        ln
    }
}
