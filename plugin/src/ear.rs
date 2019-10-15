use crate::voice::Voice;

const DEF_INPUT_TAG: &'static str = "I";

pub struct Word {
    value: f32,
    tag: String,
}

pub struct Talk {
    voice: Voice,
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
Word{value: 0., tag: DEF_INPUT_TAG.to_string()}
}

pub fn def_src() -> Src{Src::Word(def_word())}
pub fn def_ear() -> Ear{ Ear::EWord(def_word())}
