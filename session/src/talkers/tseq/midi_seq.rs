use talkers::tseq::binder::Binder;
use talkers::tseq::parser::PSequence;

struct MidiEvent {}

pub struct MidiSeq {
    current_event: usize,
    events: Vec<MidiEvent>,
}

impl MidiSeq {
    pub fn new(
        _pare: &Binder,
        _sequence: &PSequence,
        _bpm: usize,
    ) -> Result<MidiSeq, failure::Error> {
        // TODO : create midi events
        Ok(MidiSeq {
            current_event: 0,
            events: Vec::new(),
        })
    }
}
