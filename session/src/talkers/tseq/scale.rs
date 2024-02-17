use std::collections::HashMap;
use std::str::FromStr;

pub trait Scale {
    fn fetch_frequency(&self, pitch: &str) -> Option<f32>;
    fn frequency_ratio(&self, interval: i32) -> f32;
}
pub type RScale = Box<dyn Scale>;

const TEMPERED_SCALE_FREQ_0: f64 = 8.175799;
pub const TEMPERED_SCALE_PITCH_FREQ_LEN:usize = 144;
const TEMPERED_SCALE_PITCH_FREQ: [(&'static str, f32); TEMPERED_SCALE_PITCH_FREQ_LEN] = [
    ("cm1", 8.176),
    ("Cm1", 8.662),
    ("dm1", 9.177),
    ("Dm1", 9.723),
    ("em1", 10.301),
    ("fm1", 10.913),
    ("Fm1", 11.562),
    ("gm1", 12.250),
    ("Gm1", 12.978),
    ("am1", 13.750),
    ("Am1", 14.568),
    ("bm1", 15.434),
    ("c0", 16.351),
    ("C0", 17.323),
    ("d0", 18.354),
    ("D0", 19.445),
    ("e0", 20.601),
    ("f0", 21.826),
    ("F0", 23.124),
    ("g0", 24.499),
    ("G0", 25.956),
    ("a0", 27.500),
    ("A0", 29.135),
    ("b0", 30.867),
    ("c1", 32.703),
    ("C1", 34.647),
    ("d1", 36.708),
    ("D1", 38.890),
    ("e1", 41.203),
    ("f1", 43.653),
    ("F1", 46.249),
    ("g1", 48.999),
    ("G1", 51.913),
    ("a1", 55.000),
    ("A1", 58.270),
    ("b1", 61.735),
    ("c2", 65.406),
    ("C2", 69.295),
    ("d2", 73.416),
    ("D2", 77.781),
    ("e2", 82.406),
    ("f2", 87.307),
    ("F2", 92.498),
    ("g2", 97.998),
    ("G2", 103.826),
    ("a2", 110.000),
    ("A2", 116.540),
    ("b2", 123.470),
    ("c3", 130.812),
    ("C3", 138.591),
    ("d3", 146.832),
    ("D3", 155.563),
    ("e3", 164.813),
    ("f3", 174.614),
    ("F3", 184.997),
    ("g3", 195.997),
    ("G3", 207.652),
    ("a3", 220.000),
    ("A3", 233.081),
    ("b3", 246.941),
    ("c4", 261.625),
    ("C4", 277.182),
    ("d4", 293.664),
    ("D4", 311.126),
    ("e4", 329.627),
    ("f4", 349.228),
    ("F4", 369.994),
    ("g4", 391.995),
    ("G4", 415.304),
    ("a4", 440.000),
    ("A4", 466.163),
    ("b4", 493.883),
    ("c5", 523.251),
    ("C5", 554.365),
    ("d5", 587.329),
    ("D5", 622.253),
    ("e5", 659.255),
    ("f5", 698.456),
    ("F5", 739.988),
    ("g5", 783.991),
    ("G5", 830.609),
    ("a5", 880.000),
    ("A5", 932.327),
    ("b5", 987.766),
    ("c6", 1046.502),
    ("C6", 1108.730),
    ("d6", 1174.059),
    ("D6", 1244.507),
    ("e6", 1318.510),
    ("f6", 1396.912),
    ("F6", 1479.976),
    ("g6", 1567.982),
    ("G6", 1661.218),
    ("a6", 1760.000),
    ("A6", 1864.654),
    ("b6", 1975.532),
    ("c7", 2093.004),
    ("C7", 2217.460),
    ("d7", 2344.318),
    ("D7", 2489.014),
    ("e7", 2637.020),
    ("f7", 2793.824),
    ("F7", 2959.952),
    ("g7", 3135.964),
    ("G7", 3322.436),
    ("a7", 3520.000),
    ("A7", 3729.308),
    ("b7", 3951.064),
    ("c8", 4186.008),
    ("C8", 4434.920),
    ("d8", 4698.636),
    ("D8", 4978.028),
    ("e8", 5274.040),
    ("f8", 5587.648),
    ("F8", 5919.904),
    ("g8", 6270.928),
    ("G8", 6644.872),
    ("a8", 7040.000),
    ("A8", 7458.616),
    ("b8", 7902.128),
    ("c9", 8372.016),
    ("C9", 8869.840),
    ("d9", 9397.272),
    ("D9", 9956.056),
    ("e9", 10548.080),
    ("f9", 11175.296),
    ("F9", 11839.808),
    ("g9", 12541.856),
    ("G9", 13289.744),
    ("a9", 14080.000),
    ("A9", 14917.232),
    ("b9", 15804.256),
    ("c10", 16744.032),
    ("C10", 17739.680),
    ("d10", 18794.544),
    ("D10", 19912.112),
    ("e10", 21096.160),
    ("f10", 22350.592),
    ("F10", 23679.616),
    ("g10", 25083.712),
    ("G10", 26579.488),
    ("a10", 28160.000),
    ("A10", 29834.464),
    ("b10", 31608.512),
];

pub struct TemperedScale {
    pitch_freq_map: HashMap<&'static str, f32>,
}
impl TemperedScale {
    pub fn new() -> Self {
        Self {
            pitch_freq_map: HashMap::from(TEMPERED_SCALE_PITCH_FREQ),
        }
    }
}

impl Scale for TemperedScale {
    fn fetch_frequency(&self, pitch: &str) -> Option<f32> {
        match usize::from_str(pitch) {
            Ok(idx) => Some((TEMPERED_SCALE_FREQ_0 * (idx as f64 / 12.).exp2()) as f32),
            Err(_) => self.pitch_freq_map.get(pitch).map(|f| *f),
        }
    }
    fn frequency_ratio(&self, interval: i32) -> f32 {
        (interval as f32 / 12.).exp2()
    }
}

pub fn create(scale_name: &str) -> Result<RScale, failure::Error> {
    if scale_name == "tempered" {
        Ok(Box::new(TemperedScale::new()))
    } else {
        Err(failure::err_msg(format!("Scale {} not found!", scale_name)))
    }
}

pub fn create_collection() -> Result<HashMap<&'static str, RScale>, failure::Error> {
    let mut collection = HashMap::new();

    collection.insert("tempered", create("tempered")?);
    Ok(collection)
}
