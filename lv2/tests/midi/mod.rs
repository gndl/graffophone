use lv2::urid::features::URIDMap;
use lv2::atom::Forge;
use lv2::urid::{URIDOf, URIDCache, SimpleMapper};
use lv2::midi::atom::AtomMidiEvent;
use lv2::midi::MidiMessage;
use lv2::units::units::Frame;
use lv2::atom::types::{AtomSequence, UnknownAtomSequence};
use lv2::atom::Atom;
use lv2::midi::Channel;

#[derive(URIDCache)]
pub struct MyURIDCache {
    midi: URIDOf<AtomMidiEvent>,
    frame: URIDOf<Frame>,
    sequence: URIDOf<UnknownAtomSequence>
}

#[test]
fn midi_event_sequence() {
    let mut buf = vec![0u8; 1024];

    let mapper = SimpleMapper::new();
    let map_feature = URIDMap::new(&mapper);
    let cache = MyURIDCache::new(&map_feature);

    {
        let mut forge = Forge::new(&mut buf, &cache);
        let mut seq = forge.begin_sequence();
        seq.write_event(&Frame(0), MidiMessage::NoteOn {channel: Channel::Channel1, note: 42, velocity: 69});
        seq.write_event(&Frame(42), MidiMessage::NoteOff {channel: Channel::Channel1, note: 42, velocity: 69});
    }

    let expected = &[
        56u8, 0, 0, 0, // Atom_Sequence.header.size
        3, 0, 0, 0, // Atom_Sequence.header.type_
        2, 0, 0, 0, // Atom_Sequence.body.unit
        0, 0, 0, 0, // Atom_Sequence.body.pad
        0, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (1)
        3, 0, 0, 0, // Atom_MidiEvent.body.header.size
        1, 0, 0, 0, // Atom_MidiEvent.body.header.type_
        144, // MidiMessage[0] (type & channel)
        42, // MidiMessage.note
        69, // MidiMessage.velocity
        0, 0, 0, 0, 0, // _padding_ to 64bits
        42, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (1)
        3, 0, 0, 0, // Atom_MidiEvent.body.header.size
        1, 0, 0, 0, // Atom_MidiEvent.body.header.type_
        128, // MidiMessage[0] (type & channel)
        42, // MidiMessage.note
        69, // MidiMessage.velocity
        0, 0, 0, 0, 0 // _padding_ to 64bits
    ];
    let transmuted = &buf[..expected.len()];
    assert_eq!(transmuted, expected as &[u8]);

    let atom = unsafe { &*(buf.as_ptr() as *const Atom) };
    let seq: &UnknownAtomSequence = atom.read_as(&cache).unwrap();
    let seq: &AtomSequence<Frame> = seq.with_timestamps(&cache).unwrap();

    let mut iter = seq.iter();

    let ev = iter.next().unwrap();
    assert_eq!(ev.time(), Frame(0));
    assert_eq!(ev.body().read_as::<AtomMidiEvent, _>(&cache).unwrap().message(), MidiMessage::NoteOn {channel: Channel::Channel1, note: 42, velocity: 69});

    let ev = iter.next().unwrap();
    assert_eq!(ev.time(), Frame(42));
    assert_eq!(ev.body().read_as::<AtomMidiEvent, _>(&cache).unwrap().message(), MidiMessage::NoteOff {channel: Channel::Channel1, note: 42, velocity: 69});

    assert!(iter.next().is_none());
}
