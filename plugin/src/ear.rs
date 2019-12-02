use crate::audio_talker::AudioTalker;
use crate::control_talker::ControlTalker;
use crate::horn;
use crate::talker::Talker;
use std::marker::PhantomData;
use std::rc::Rc;
//use voice::VoiceT;

pub const DEF_INPUT_TAG: &'static str = "I";

pub struct TalkT<T> {
    tag: String,
    tkr: Rc<dyn Talker>,
    port: i32,
    _marker: PhantomData<T>,
    //    horn: T,
}
pub type AudioTalk = TalkT<horn::Audio>;
pub type ControlTalk = TalkT<horn::Control>;
pub type CvTalk = TalkT<horn::Cv>;

pub struct TalksT<T> {
    tag: String,
    talks: Vec<TalkT<T>>,
    _marker: PhantomData<T>,
}

pub type AudioTalks = TalksT<horn::Audio>;
pub type ControlTalks = TalksT<horn::Control>;
pub type CvTalks = TalksT<horn::Cv>;
/*
pub struct AudioTalk
{
    pub talk: Talk,
    pub horn: horn::Audio,
}
pub struct ControlTalk {
    pub talk: Talk,
    pub horn: horn::Control,
}
pub struct CvTalk {
    pub talk: Talk,
    pub horn: horn::Cv,
}
pub enum Src {
    Word(Word),
    Talk(Talk),
}
pub struct Bin {
    src: Src,
}
pub struct AudioTalks {
    tag: String,
    talks: Vec<AudioTalk>,
}
pub struct ControlTalks {
    tag: String,
    talks: Vec<ControlTalk>,
}
pub struct CvTalks {
    tag: String,
    talks: Vec<CvTalk>,
}
pub struct Bins {
    bins: Vec<Bin>,
    tag: String,
}
*/
pub enum Ear {
    Audio(AudioTalk),
    Control(ControlTalk),
    Cv(CvTalk),
    Audios(AudioTalks),
    Controls(ControlTalks),
    Cvs(CvTalks),
}

pub fn def_audio(tag: Option<String>, value: Option<f32>) -> AudioTalk {
    AudioTalk {
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: Rc::new(AudioTalker::new(value, Some(true))),
        port: 0,
        _marker: PhantomData,
    }
}
pub fn def_control(tag: Option<String>, value: Option<f32>) -> ControlTalk {
    ControlTalk {
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: Rc::new(ControlTalker::new(value, Some(true))),
        port: 0,
        _marker: PhantomData,
    }
}
/*
pub fn def_src() -> Src {
    Src::Word(def_word())
}
 */
pub fn def_ear() -> Ear {
    Ear::Control(def_control(None, None))
}

pub fn control(tag: Option<String>, value: Option<f32>) -> ControlTalk {
    def_control(tag, value)
}

pub fn audio(
    tag: Option<String>,
    value: Option<f32>,
    talker_port: Option<(&Rc<dyn Talker>, i32)>,
) -> AudioTalk {
    match value {
        Some(_v) => def_audio(tag, value),
        None => match talker_port {
            Some((tkr, port)) => AudioTalk {
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
                tkr: Rc::clone(tkr),
                port: port,
                _marker: PhantomData,
            },
            None => def_audio(tag, None),
        },
    }
}
/*
pub fn mk_bin(tag: Option<String>, src: Option<Src>, value: Option<f32>) -> Bin {
    match src {
        Some(src) => Bin { src },
        None => Bin {
            src: Src::Word(mk_word(tag, value)),
        },
    }
}

pub fn mk_word_bin(tag: Option<String>, value: f32) -> Bin {
    Bin {
        src: Src::Word(Word {
            value: Rc::new(CellBuffer::new(value)),
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

pub fn talk_of_ear (ear: &Ear) -> Option<Ear> {

      match ear {
      EWord(_) => None, EWords (_) => None,
      ETalk (talk) => Some(talk),
      EBin (bin) =>
              match bin.src { Talk (talk) -> Some(talk), Word (_ ) => None,
              },
      ETalks( ets) => L.rev_append (A.to_list efs.talks) talks
      | EBins ebs -> A.fold_right ebs.bins ~init:talks ~f:(fun bin talks ->
          match bin.src with Talk talk -> talk::talks | Word _ -> talks)
    )

    pub fn talks_of_ears ears =

pub fn listen_voice<T>(talk: &TalkT<T>, tick: i64, len: usize) -> VoiceT<T> {
    let port = talk.port;
    let voice = talk.tkr.voices().get(port);

    if tick != voice.tick()
  || len > voice.len()
  {
    talk.tkr.talk (port tick len);
  }
    voice
}
*/
