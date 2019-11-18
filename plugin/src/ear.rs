use crate::hidden_constant_talker::HiddenConstantTalker;
use crate::talker::Talker;
use std::rc::Rc;

pub const DEF_INPUT_TAG: &'static str = "I";

pub struct Word {
    value: f32,
    tag: String,
}
pub struct Talk {
    tkr: Rc<dyn Talker>,
    port: i32,
    tag: String,
}
pub enum Src {
    Word(Word),
    Talk(Talk),
}
pub struct Bin {
    src: Src,
}
pub struct Words {
    words: Vec<Word>,
    tag: String,
}
pub struct Talks {
    talks: Vec<Talk>,
    tag: String,
}
pub struct Bins {
    bins: Vec<Bin>,
    tag: String,
}
pub enum Ear {
    EWord(Word),
    ETalk(Talk),
    EBin(Bin),
    EWords(Words),
    ETalks(Talks),
    EBins(Bins),
}

pub fn def_word() -> Word {
    Word {
        value: 0.,
        tag: DEF_INPUT_TAG.to_string(),
    }
}

pub fn def_src() -> Src {
    Src::Word(def_word())
}
pub fn def_ear() -> Ear {
    Ear::EWord(def_word())
}
fn mk_constant_talk(tag: Option<String>, value: Option<f32>) -> Talk {
    Talk {
        tkr: Rc::new(HiddenConstantTalker::new(value)),
        port: 0,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
    }
}

pub fn mk_word(tag: Option<String>, value: Option<f32>) -> Word {
    Word {
        value: value.unwrap_or(0.),
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
    }
}

pub fn mk_talk(
    tag: Option<String>,
    value: Option<f32>,
    talker_port: Option<(&Rc<dyn Talker>, i32)>,
) -> Talk {
    match value {
        Some(_v) => mk_constant_talk(tag, value),
        None => match talker_port {
            Some((tkr, port)) => Talk {
                tkr: Rc::clone(tkr),
                port: port,
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
            },
            None => mk_constant_talk(tag, None),
        },
    }
}

pub fn mk_bin(tag: Option<String>, src: Option<Src>, value: Option<f32>) -> Bin {
    match src {
        Some(src) => Bin { src },
        None => Bin {
            src: Src::Word(Word {
                value: value.unwrap_or(0.),
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
            }),
        },
    }
}

pub fn mk_word_bin(tag: Option<String>, value: f32) -> Bin {
    Bin {
        src: Src::Word(Word {
            value,
            tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        }),
    }
}

pub fn mk_talk_bin(tag: Option<String>, tkr: &Rc<dyn Talker>, port: i32) -> Bin {
    Bin {
        src: Src::Talk(Talk {
            tkr: Rc::clone(tkr),
            port,
            tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        }),
    }
}

pub fn mk_words(tag: Option<String>) -> Words {
    Words {
        words: Vec::new(),
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
    }
}

pub fn mk_talks(tag: Option<String>) -> Talks {
    Talks {
        talks: Vec::new(),
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
    }
}

pub fn mk_bins(tag: Option<String>) -> Bins {
    Bins {
        bins: Vec::new(),
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
    }
}
