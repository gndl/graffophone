use std::collections::HashMap;
use std::str::FromStr;


pub struct Scale {
    pub name: &'static str,
    freq_0: f64,
    pitchs_name_ratio: Vec<(&'static str, f64)>,
}
impl Scale {
    pub fn new(name: &'static str, freq_0: f64, pitchs_id_ratio: Vec<(&'static str, f64)>,) -> Self {
        Self {
            name,
            freq_0,
            pitchs_name_ratio: pitchs_id_ratio,
        }
    }

    pub fn fetch_frequency(&self, pitch: &str) -> Result<f32, failure::Error> {
        match pitch.rfind(|c: char| !c.is_ascii_digit()) {
            Some(p) => {
                match f32::from_str(pitch) {
                    Ok(f) => Ok(f), // pitch is a float, this is the frequency
                    Err(_) => {
                        // pitch is alfanumeric, this is the pitch id (name + octave)
                        let octave_pos = p + 1;
                        let pitch_name = &pitch[..octave_pos];

                        if let Some(octave_str) = pitch.get(octave_pos..) {
                            match f64::from_str(octave_str) {
                                Ok(octave) => {
                                    for (name, ratio) in &self.pitchs_name_ratio {
                                        if *name == pitch_name {
                                            let f = (self.freq_0 * octave.exp2() * *ratio) as f32;
                                            return Ok(f as f32);
                                        }
                                    }
                                }
                                Err(_) => (),
                            }
                        }
                        Err(failure::err_msg(format!("Tseq pitch {} not found!", pitch)))
                    }
                }
            },
            None => {
                // pitch is purely digital, this is the midi pitch number
                let num = usize::from_str(pitch).unwrap();
                let pitchs_per_octave = self.pitchs_name_ratio.len();
                let octave = (num / pitchs_per_octave) as f64 - 1.;
                let idx = num % pitchs_per_octave;

                let (_, ratio) = &self.pitchs_name_ratio[idx];
                let f = self.freq_0 * octave.exp2() * *ratio;
                Ok(f as f32)
            }
        }
    }
    pub fn frequency_ratio(&self, interval: i32) -> f32 {
        (interval as f32 / 12.).exp2()
    }
}


pub fn create_tempered_scale() -> Scale {
    let freq_0 = 440. as f64 / (57. / 12. as f64).exp2();

    Scale::new("tempered", freq_0,
        vec![
            ("C", 1.),
            ("C#", (1. / 12. as f64).exp2()),
            ("D", (2. / 12. as f64).exp2()),
            ("D#", (3. / 12. as f64).exp2()),
            ("E", (4. / 12. as f64).exp2()),
            ("F", (5. / 12. as f64).exp2()),
            ("F#", (6. / 12. as f64).exp2()),
            ("G", (7. / 12. as f64).exp2()),
            ("G#", (8. / 12. as f64).exp2()),
            ("A", (9. / 12. as f64).exp2()),
            ("A#", (10. / 12. as f64).exp2()),
            ("B", (11. / 12. as f64).exp2()),
            ])
}

pub fn create_natural_scale() -> Scale {
    let freq_0 = (440. as f64 * 3.) / (5. as f64 * 16.);

    Scale::new("natural", freq_0,
        vec![
            ("C", 1.),
            ("C#", 25. / 24. as f64),
            ("D", 9. / 8. as f64),
            ("D#", 6. / 5. as f64),
            ("E", 5. / 4. as f64),
            ("F", 4. / 3. as f64),
            ("F#", 25. / 18. as f64),
            ("G", 3. / 2. as f64),
            ("G#", 25. / 16. as f64),
            ("A", 5. / 3. as f64),
            ("A#", 16. / 9. as f64),
            ("B", 15. / 8. as f64),
            ])
}

pub fn create_collection<'a>() -> Result<HashMap<&'static str, Scale>, failure::Error> {
    let mut collection = HashMap::new();

    collection.insert("tempered", create_tempered_scale());
    collection.insert("natural", create_natural_scale());

    Ok(collection)
}
