use crate::audio_talker::AudioTalker;
use crate::control_talker::ControlTalker;
use crate::cv_talker::CvTalker;
use crate::talker::Talker;
use std::cell::RefCell;
use std::rc::Rc;
use voice::PortType;

pub const DEF_INPUT_TAG: &'static str = "I";

pub struct Talk {
    port_type: PortType,
    tag: String,
    tkr: Rc<RefCell<dyn Talker>>,
    port: usize,
}

pub struct Talks {
    port_type: PortType,
    tag: String,
    talks: Vec<Talk>,
}

pub enum Ear {
    Talk(Talk),
    Talks(Talks),
}

pub fn def_audio(tag: Option<String>, value: Option<f32>) -> Ear {
    Ear::Talk(Talk {
        port_type: PortType::Audio,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: Rc::new(RefCell::new(AudioTalker::new(value, Some(true)))),
        port: 0,
    })
}
pub fn def_control(tag: Option<String>, value: Option<f32>) -> Ear {
    Ear::Talk(Talk {
        port_type: PortType::Control,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: Rc::new(RefCell::new(ControlTalker::new(value, Some(true)))),
        port: 0,
    })
}
pub fn def_cv(tag: Option<String>, value: Option<f32>) -> Ear {
    Ear::Talk(Talk {
        port_type: PortType::Cv,
        tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
        tkr: Rc::new(RefCell::new(CvTalker::new(value, Some(true)))),
        port: 0,
    })
}
/*
pub fn def_src() -> Src {
    Src::Word(def_word())
}
 */
pub fn def_ear() -> Ear {
    def_control(None, None)
}

pub fn control(tag: Option<String>, value: Option<f32>) -> Ear {
    def_control(tag, value)
}

pub fn audio(
    tag: Option<String>,
    value: Option<f32>,
    talker_port: Option<(&Rc<RefCell<dyn Talker>>, usize)>,
) -> Ear {
    match value {
        Some(_v) => def_audio(tag, value),
        None => match talker_port {
            Some((tkr, port)) => Ear::Talk(Talk {
                port_type: PortType::Audio,
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
                tkr: Rc::clone(tkr),
                port: port,
            }),
            None => def_audio(tag, None),
        },
    }
}
pub fn cv(
    tag: Option<String>,
    value: Option<f32>,
    talker_port: Option<(&Rc<RefCell<dyn Talker>>, usize)>,
) -> Ear {
    match value {
        Some(_v) => def_cv(tag, value),
        None => match talker_port {
            Some((tkr, port)) => Ear::Talk(Talk {
                port_type: PortType::Cv,
                tag: tag.unwrap_or(DEF_INPUT_TAG.to_string()),
                tkr: Rc::clone(tkr),
                port: port,
            }),
            None => def_cv(tag, None),
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
*/

pub fn listen_talk(talk: &Talk, tick: i64, len: usize) -> usize {
    let need_talking;
    let port = talk.port;
    {
        let mut tkr = talk.tkr.borrow_mut();
        let voice = tkr.voices().get(port).unwrap().borrow();
        need_talking = tick != voice.tick() || len > voice.len();
    }
    if need_talking {
        let mut tkr/*: RefMut<_> */= talk.tkr.borrow_mut();
        tkr.talk(port, tick, len)
    } else {
        len
    }
}

pub fn listen(ear: &Ear, tick: i64, len: usize) -> usize {
    match ear {
        Ear::Talk(talk) => listen_talk(&talk, tick, len),
        Ear::Talks(talks) => {
            let mut ln = len;

            for tlk in &talks.talks {
                ln = listen_talk(&tlk, tick, ln);
            }
            ln
        }
    }
}
