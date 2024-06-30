use std::collections::HashMap;

use scale::{self, Scale};

pub struct FreqPitch {
    pub freq: f32,
    pub pitch: String,
}

pub struct PitchFetcher {
    pub name: &'static str,
    freqs_pitch: Vec<FreqPitch>,
}
impl PitchFetcher {
    pub fn new(scale: &Scale,) -> Self {
        let pitchs_names = scale.get_pitchs_names();
        let mut freqs_pitch = Vec::with_capacity(pitchs_names.len() * 11);

        freqs_pitch.push(FreqPitch{freq: f32::MIN, pitch: "MIN".to_string()});

        for octave in 0..11 {
            for name in &pitchs_names {
                let pitch = format!("{}{}", name, octave);
                let freq = scale.fetch_frequency(&pitch).expect("Pitch error");

                freqs_pitch.push(FreqPitch{freq, pitch});
            }
        }

        freqs_pitch.push(FreqPitch{freq: f32::MAX, pitch: "MAX".to_string()});

        Self {
            name: scale.name,
            freqs_pitch,
        }
    }


    pub fn fetch_pitch(&self, freq: f32) -> Result<String, failure::Error> {
        let mut left = 0;
        let mut right = self.freqs_pitch.len() - 1;
        let mut m = right / 2;

        while left <= right {
            if self.freqs_pitch[m].freq == freq {
                return Ok(self.freqs_pitch[m].pitch.clone());
            }
            else if self.freqs_pitch[m].freq > freq {
                right = m - 1;
            }
            else {
                left = m + 1;
            }

            m = (left + right) / 2;
        }

        if freq - self.freqs_pitch[m].freq < self.freqs_pitch[m + 1].freq - freq {
            return Ok(self.freqs_pitch[m].pitch.clone());
        }
        else {
            return Ok(self.freqs_pitch[m + 1].pitch.clone());
        }
    }
}


pub struct Collection {
    map: HashMap<&'static str, PitchFetcher>,
}
impl Collection {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        let scale_col = scale::Collection::new();

        for scale in scale_col.values() {
            map.insert(scale.name, PitchFetcher::new(scale));
        }

        Self {
            map,
        }
    }

    pub fn fetch<'a>(&'a self, scale_name: &str) -> Result<&'a PitchFetcher, failure::Error> {
        match self.map.get(scale_name) {
            Some(pf) => Ok(pf),
            None => Err(failure::err_msg(format!("Tseq scale {} unknown!", scale_name))),
        }
    }

    pub fn default<'a>(&'a self) -> &'a PitchFetcher {
        self.map.get(scale::DEFAULT).unwrap()
    }
}
