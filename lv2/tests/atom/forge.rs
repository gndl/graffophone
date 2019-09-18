use lv2::atom::*;
use lv2::atom::types::*;

use lv2::urid::features::URIDMap;
use lv2::urid::{URID, URIDOf, URIDCache, SimpleMapper};
use lv2::units::units::Frame;

#[derive(URIDCache)]
pub struct BasicURIDCache {
    float_urid: URIDOf<AtomFloat>,
    double_urid: URIDOf<AtomDouble>,
    frame: URIDOf<Frame>,
    sequence: URIDOf<UnknownAtomSequence>
}

#[test]
fn test_forge() {
    let mut buf = vec![0u8; 256];
    let mapper = SimpleMapper::new();
    let map_feature = URIDMap::new(&mapper);
    let cache = BasicURIDCache::new(&map_feature);
    let forge = Forge::new(&mut buf, &cache);

    assert_eq!(forge.get_urid::<AtomFloat>(), URID::new(1).unwrap());
    assert_eq!(forge.get_urid::<AtomDouble>(), URID::new(2).unwrap());
}

#[test]
fn test_deep_sequence() {
    let mut buf = vec![0u8; 256];
    let mapper = SimpleMapper::new();
    let map_feature = URIDMap::new(&mapper);
    let cache = BasicURIDCache::new(&map_feature);
    {
        let mut forge = Forge::new(&mut buf, &cache);
        {
            let mut seq1 = forge.begin_sequence();
            {
                let mut seq2 = seq1.begin_sequence(&Frame(2));
                {
                    let mut seq3 = seq2.begin_sequence(&Frame(4));
                    {
                        let mut seq4 = seq3.begin_sequence(&Frame(6));
                        seq4.write_event(&Frame(8), 42.0);
                    }
                    seq3.write_event(&Frame(10), 42.0);
                }
            }
        }
    }

    eprintln!("{:?}", buf);
    let expected = &[
        128u8, 0, 0, 0, // Atom_Sequence.header.size
        4, 0, 0, 0, // Atom_Sequence.header.type_
        3, 0, 0, 0, // Atom_Sequence.body.unit
        0, 0, 0, 0, // Atom_Sequence.body.pad
        2, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (1)
            104, 0, 0, 0, // Atom_Sequence.header.size
            4, 0, 0, 0, // Atom_Sequence.header.type_
            3, 0, 0, 0, // Atom_Sequence.body.unit
            0, 0, 0, 0, // Atom_Sequence.body.pad
            4, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (2)
                80, 0, 0, 0, // Atom_Sequence.header.size
                4, 0, 0, 0, // Atom_Sequence.header.type_
                3, 0, 0, 0, // Atom_Sequence.body.unit
                0, 0, 0, 0, // Atom_Sequence.body.pad
                6, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (3)
                    32, 0, 0, 0, // Atom_Sequence.header.size
                    4, 0, 0, 0, // Atom_Sequence.header.type_
                    3, 0, 0, 0, // Atom_Sequence.body.unit
                    0, 0, 0, 0, // Atom_Sequence.body.pad
                    8, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (4)
                        4, 0, 0, 0, // Atom_Float.header.size
                        1, 0, 0, 0, // Atom_Float.header.type_
                        0, 0, 40, 66, // Atom_Float.body
                        0, 0, 0, 0, // _padding_ to 64bits
                    10, 0, 0, 0, 0, 0, 0, 0, // Atom_Event.time (4)
                    4, 0, 0, 0, // Atom_Float.header.size
                    1, 0, 0, 0, // Atom_Float.header.type_
                    0, 0, 40, 66, // Atom_Float.body
                    0, 0, 0, 0, // _padding_ to 64bits
    ];
    let transmuted = &buf[..expected.len()];
    assert_eq!(transmuted, expected as &[u8]);
}
