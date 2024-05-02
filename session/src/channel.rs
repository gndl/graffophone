
pub struct Layout{}

pub const DEFAULT_LAYOUT: &str = "stereo";
pub const LAYOUTS: [&str; 30] = [
    "mono",
    DEFAULT_LAYOUT,
    "2.1",
    "3.0",
    "3.0(back)",
    "4.0",
    "quad",
    "quad(side)",
    "3.1",
    "5.0",
    "5.0(side)",
    "4.1",
    "5.1",
    "5.1(side)",
    "6.0",
    "6.0(front)",
    "hexagonal",
    "6.1",
    "6.1(back)",
    "6.1(front)",
    "7.0",
    "7.0(front)",
    "7.1",
    "7.1(wide)",
    "7.1(wide-side)",
    "7.1(top)",
    "octagonal",
    "cube",
    "hexadecagonal",
    "22.2"
];

impl Layout {
    
    fn definitions() -> Vec<Vec<&'static str>> {
        vec![
            vec!["FC"], // mono
            vec!["FL", "FR"], // stereo
            vec!["FL", "FR", "LFE"], // 2.1
            vec!["FL", "FR", "FC"], // 3.0
            vec!["FL", "FR", "BC"], // 3.0(back)
            vec!["FL", "FR", "FC", "BC"], // 4.0
            vec!["FL", "FR", "BL", "BR"], // quad
            vec!["FL", "FR", "SL", "SR"], // quad(side)
            vec!["FL", "FR", "FC", "LFE"], // 3.1
            vec!["FL", "FR", "FC", "BL", "BR"], // 5.0
            vec!["FL", "FR", "FC", "SL", "SR"], // 5.0(side)
            vec!["FL", "FR", "FC", "LFE", "BC"], // 4.1
            vec!["FL", "FR", "FC", "LFE", "BL", "BR"], // 5.1
            vec!["FL", "FR", "FC", "LFE", "SL", "SR"], // 5.1(side)
            vec!["FL", "FR", "FC", "BC", "SL", "SR"], // 6.0
            vec!["FL", "FR", "FLC", "FRC", "SL", "SR"], // 6.0(front)
            vec!["FL", "FR", "FC", "BL", "BR", "BC"], // hexagonal
            vec!["FL", "FR", "FC", "LFE", "BC", "SL", "SR"], // 6.1
            vec!["FL", "FR", "FC", "LFE", "BL", "BR", "BC"], // 6.1(back)
            vec!["FL", "FR", "LFE", "FLC", "FRC", "SL", "SR"], // 6.1(front)
            vec!["FL", "FR", "FC", "BL", "BR", "SL", "SR"], // 7.0
            vec!["FL", "FR", "FC", "FLC", "FRC", "SL", "SR"], // 7.0(front)
            vec!["FL", "FR", "FC", "LFE", "BL", "BR", "SL", "SR"], // 7.1
            vec!["FL", "FR", "FC", "LFE", "BL", "BR", "FLC", "FRC"], // 7.1(wide)
            vec!["FL", "FR", "FC", "LFE", "FLC", "FRC", "SL", "SR"], // 7.1(wide-side)
            vec!["FL", "FR", "FC", "LFE", "BL", "BR", "TFL", "TFR"], // 7.1(top)
            vec!["FL", "FR", "FC", "BL", "BR", "BC", "SL", "SR"], // octagonal
            vec!["FL", "FR", "BL", "BR", "TFL", "TFR", "TBL", "TBR"], // cube
            vec!["FL", "FR", "FC", "BL", "BR", "BC", "SL", "SR", "TFL", "TFC", "TFR", "TBL", "TBC", "TBR", "WL", "WR"], // hexadecagonal
            vec!["FL", "FR", "FC", "LFE", "BL", "BR", "FLC", "FRC", "BC", "SL", "SR", "TC", "TFL", "TFC", "TFR", "TBL", "TBC", "TBR", "LFE2", "TSL", "TSR", "BFC", "BFL", "BFR"], // 22.2
        ]
    }
    
    pub fn names() -> &'static[&'static str] {
        &LAYOUTS
    }

    pub fn index(layout: &str) -> usize {
        for idx in 0..LAYOUTS.len() {
            if LAYOUTS[idx] == layout {
                return idx;
            }
        }
        eprintln!("Unknow channel layout {}. Fallback to {}.", layout, DEFAULT_LAYOUT);
        1
    }
    
    pub fn from_index(index: usize) -> &'static str {
        LAYOUTS[index]
    }

    pub fn channels_names(layout: &str) -> Vec<&'static str> {
        let mut idx = 0;

        for channels_names in Layout::definitions() {
            if LAYOUTS[idx] == layout {
                return channels_names;
            }
            idx += 1;
        }
        eprintln!("Unknow channel layout {}. Fallback to {}.", layout, DEFAULT_LAYOUT);
        vec!["Left", "Right"]
    }
    
    pub fn channels(layout: &str) -> usize {
        Layout::channels_names(layout).len()
    }
    
    pub fn from_channels(channels: usize) -> &'static str {
        for (idx, channels_names) in Layout::definitions().iter().enumerate() {
            if channels_names.len() == channels {
                return  LAYOUTS[idx];
            }
        }
        eprintln!("Unknow channel layout with {} channels. Fallback to {}.", channels, DEFAULT_LAYOUT);
        DEFAULT_LAYOUT
    }

    pub fn channels_names_from_channels(channels: usize) -> Vec<&'static str> {
        for channels_names in Layout::definitions() {
            if channels_names.len() == channels {
                return  channels_names;
            }
        }
        eprintln!("Unknow channel layout with {} channels. Fallback to {}.", channels, DEFAULT_LAYOUT);
        vec!["Left", "Right"]
    }
    
}