use talker::ctalker;
use talker::dsp;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const SUM_MODEL: &str = "Sum";

const AUDIO_VOICE_PORT: usize = 1;

pub struct Sum {}
impl Sum {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(SUM_MODEL, SUM_MODEL);

        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_voice(voice::cv(Some("cv"), 0.));
        base.add_voice(voice::audio(Some("au"), 0.));

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
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(PRODUCT_MODEL, PRODUCT_MODEL);

        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_voice(voice::cv(Some("cv"), 0.));
        base.add_voice(voice::audio(Some("au"), 0.));

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
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(AVERAGE_MODEL, AVERAGE_MODEL);

        base.add_ear(ear::cv(None, -20000., 20000., 0., &Init::DefValue)?);

        base.add_voice(voice::cv(Some("cv"), 0.));
        base.add_voice(voice::audio(Some("au"), 0.));

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

pub const TANH_SUM_MODEL: &str = "tanhSum";
pub struct TanhSum {}
impl TanhSum {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(TANH_SUM_MODEL, TANH_SUM_MODEL);

        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_voice(voice::audio(Some("au"), 0.));

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Mathematics", TANH_SUM_MODEL, TANH_SUM_MODEL)
    }
}

impl Talker for TanhSum {
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

        dsp::audioize_buffer_by_tanh(voice_buf, 0, ln);
        ln
    }
}
