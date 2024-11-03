use talker::ctalker;
use talker::dsp;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;

pub const SUM_MODEL: &str = "Sum";

const AUDIO_VOICE_PORT: usize = 1;

pub struct Sum {}
impl Sum {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", SUM_MODEL, SUM_MODEL)
    }
}

impl Talker for Sum {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let inputs_ear = base.ear(0);
        let voice_buf = base.voice(port).cv_buffer();

        if inputs_ear.sets_len() == 0 {
            voice_buf.fill(0.);
        }
        else {
            let input_buf = inputs_ear.get_set_hum_cv_buffer(0, 0);

            for i in 0..ln {
                voice_buf[i] = input_buf[i];
            }

            for input_idx in 1..inputs_ear.sets_len() {
                let input_buf = inputs_ear.get_set_hum_cv_buffer(input_idx, 0);

                for i in 0..ln {
                    voice_buf[i] = voice_buf[i] + input_buf[i];
                }
            }
        }

        if port == AUDIO_VOICE_PORT {
            dsp::audioize_buffer_by_clipping(voice_buf, 0, ln);
        }
        ln
    }
}

pub const PRODUCT_MODEL: &str = "Product";

pub struct Product {}
impl Product {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", PRODUCT_MODEL, PRODUCT_MODEL)
    }
}

impl Talker for Product {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let inputs_ear = base.ear(0);
        let voice_buf = base.voice(port).cv_buffer();

        if inputs_ear.sets_len() == 0 {
            voice_buf.fill(0.);
        }
        else {
            let input_buf = inputs_ear.get_set_hum_cv_buffer(0, 0);

            for i in 0..ln {
                voice_buf[i] = input_buf[i];
            }

            for input_idx in 1..inputs_ear.sets_len() {
                let input_buf = inputs_ear.get_set_hum_cv_buffer(input_idx, 0);

                for i in 0..ln {
                    voice_buf[i] = voice_buf[i] * input_buf[i];
                }
            }
        }

        if port == AUDIO_VOICE_PORT {
            dsp::audioize_buffer_by_clipping(voice_buf, 0, ln);
        }
        ln
    }
}

pub const AVERAGE_MODEL: &str = "Average";

pub struct Average {}
impl Average {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.add_ear(ear::cv(None, -20000., 20000., 0., &Init::DefValue)?);

        base.add_cv_voice(Some("cv"), 0.);
        base.add_audio_voice(Some("au"), 0.);

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Modulator", AVERAGE_MODEL, AVERAGE_MODEL)
    }
}

impl Talker for Average {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let input_buf = base.ear_cv_buffer(0);
        let voice_buf = base.voice(port).cv_buffer();

        for i in 0..ln {
            voice_buf[i] = input_buf[i];
        }

        if port == AUDIO_VOICE_PORT {
            dsp::audioize_buffer_by_clipping(voice_buf, 0, ln);
        }

        ln
    }
}

pub struct SumAudioizer {}
impl SumAudioizer {
    pub fn new(base: &mut TalkerBase) -> Result<SumAudioizer, failure::Error> {
        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_ear(ear::cv(Some("gain"), -100., 100., 1., &Init::DefValue)?);

        base.add_audio_voice(Some("au"), 0.);

        Ok(Self{})
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", TANH_SUM_MODEL, TANH_SUM_MODEL)
    }

    pub fn talk<F>(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize, audioizer: F) -> usize
    where F: Fn(&mut [f32], usize, usize),
    {
        let ln = base.listen(tick, len);
        let inputs_ear = base.ear(0);
        let gain_buf = base.ear_cv_buffer(1);
        let voice_buf = base.voice(port).cv_buffer();

        if inputs_ear.sets_len() == 0 {
            voice_buf.fill(0.);
        }
        else {
            let input_buf = inputs_ear.get_set_hum_cv_buffer(0, 0);

            for i in 0..ln {
                voice_buf[i] = input_buf[i];
            }

            for input_idx in 1..inputs_ear.sets_len() {
                let input_buf = inputs_ear.get_set_hum_cv_buffer(input_idx, 0);

                for i in 0..ln {
                    voice_buf[i] = voice_buf[i] + input_buf[i];
                }
            }
            for i in 0..ln {
                voice_buf[i] = voice_buf[i] * gain_buf[i];
            }
        }

        audioizer(voice_buf, 0, ln);
        ln
    }
}

pub const ATAN_SUM_MODEL: &str = "AtanSum";
pub struct AtanSum {
    sum_audioizer: SumAudioizer,
}
impl AtanSum {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let sum_audioizer = SumAudioizer::new(&mut base)?;

        Ok(ctalker!(base, Self {sum_audioizer}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", ATAN_SUM_MODEL, ATAN_SUM_MODEL)
    }
}

impl Talker for AtanSum {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.sum_audioizer.talk(base, port, tick, len, dsp::audioize_buffer_by_atan)
    }
}

pub const TANH_SUM_MODEL: &str = "TanhSum";
pub struct TanhSum {
    sum_audioizer: SumAudioizer,
}
impl TanhSum {
    pub fn new(mut base: TalkerBase) -> Result<CTalker, failure::Error> {
        let sum_audioizer = SumAudioizer::new(&mut base)?;

        Ok(ctalker!(base, Self {sum_audioizer}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", TANH_SUM_MODEL, TANH_SUM_MODEL)
    }
}

impl Talker for TanhSum {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        self.sum_audioizer.talk(base, port, tick, len, dsp::audioize_buffer_by_tanh)
    }
}
