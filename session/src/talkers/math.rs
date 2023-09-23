use talker::ctalker;
use talker::ear;
use talker::ear::Ear;
use talker::ear::Init;
use talker::ear::Set;
use talker::horn::PortType;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use talker::voice;

pub const SUM_MODEL: &str = "Sum";

pub struct Sum {}
impl Sum {
    pub fn new() -> Result<CTalker, failure::Error> {
        let mut base = TalkerBase::new(SUM_MODEL, SUM_MODEL);

        let stem_set =
            Set::from_attributs(&vec![("", PortType::Cv, -20000., 20000., 0., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_voice(voice::cv(None, 0.));

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Mathematics", SUM_MODEL, SUM_MODEL)
    }
}

impl Talker for Sum {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let inputs_ear = base.ear(0);
        let voice_buf = base.voice(port).cv_buffer();

        for i in 0..ln {
            voice_buf[i] = 0.;
        }

        for input_idx in 0..inputs_ear.sets_len() {
            let input_buf = inputs_ear.get_set_hum_cv_buffer(input_idx, 0);

            for i in 0..ln {
                voice_buf[i] = voice_buf[i] + input_buf[i];
            }
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
            Set::from_attributs(&vec![("", PortType::Cv, -10000., 10000., 1., Init::Empty)])?;

        base.add_ear(Ear::new(None, true, Some(stem_set), None));

        base.add_voice(voice::cv(None, 0.));

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Mathematics", PRODUCT_MODEL, PRODUCT_MODEL)
    }
}

impl Talker for Product {
    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {
        let ln = base.listen(tick, len);
        let inputs_ear = base.ear(0);
        let voice_buf = base.voice(port).cv_buffer();

        for i in 0..ln {
            voice_buf[i] = 1.;
        }

        for input_idx in 0..inputs_ear.sets_len() {
            let input_buf = inputs_ear.get_set_hum_cv_buffer(input_idx, 0);

            for i in 0..ln {
                voice_buf[i] = voice_buf[i] * input_buf[i];
            }
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

        base.add_voice(voice::cv(None, 0.));

        Ok(ctalker!(base, Self {}))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::new("Modulator", AVERAGE_MODEL, AVERAGE_MODEL)
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

        ln
    }
}
