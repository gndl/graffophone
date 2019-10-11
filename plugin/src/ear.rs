use crate::voice::Voice;

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

pub enum Ear {
    EWord(Word),
    ETalk(Talk),
    EBin(Bin),
}
