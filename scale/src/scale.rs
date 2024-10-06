use std::collections::HashMap;
use std::str::FromStr;

pub const DEFAULT: &str = "12ET";

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

    pub fn get_pitchs_names(&self) -> Vec<&'static str> {
        let mut pitchs_names = Vec::with_capacity(self.pitchs_name_ratio.len());

        for (name, _) in &self.pitchs_name_ratio {
            pitchs_names.push(*name);
        }
        pitchs_names
    }

    pub fn pitch_name_to_number(&self, pitch: &str) -> Result<usize, failure::Error> {
        match pitch.rfind(|c: char| !c.is_ascii_digit()) {
            Some(p) => {
                match f32::from_str(pitch) {
                    Ok(_) => Err(failure::err_msg(format!("Tseq pitch frequency {} has not number!", pitch))),
                    Err(_) => {
                        // pitch is alphanumeric, this is the pitch id (name + octave)
                        let octave_pos = p + 1;
                        let pitch_name = &pitch[..octave_pos];

                        if let Some(octave_str) = pitch.get(octave_pos..) {
                            match usize::from_str(octave_str) {
                                Ok(octave) => {
                                    for (idx, (name, _)) in self.pitchs_name_ratio.iter().enumerate() {
                                        if *name == pitch_name {
                                            // octave is increased by 1 to match MIDI numbers
                                            let num = self.pitchs_name_ratio.len() * (octave + 1) + idx;
                                            return Ok(num);
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
                Ok(usize::from_str(pitch).unwrap())
            }
        }
    }

    pub fn pitch_number_to_name(&self, number: usize) -> String {
        let pitchs_per_octave = self.pitchs_name_ratio.len();
        let octave = (number / pitchs_per_octave) - 1;
        let idx = number % pitchs_per_octave;

        let (name, _) = &self.pitchs_name_ratio[idx];
        format!("{}{}", name, octave)
    }

    pub fn pitch_number_to_frequency(&self, number: usize) -> f32 {
        let pitchs_per_octave = self.pitchs_name_ratio.len();
        let octave = ((number / pitchs_per_octave) - 1) as f64;
        let idx = number % pitchs_per_octave;

        let (_, ratio) = &self.pitchs_name_ratio[idx];
        let f = self.freq_0 * octave.exp2() * *ratio;
        f as f32
    }
    
    pub fn pitch_name_to_frequency(&self, pitch: &str) -> Result<f32, failure::Error> {
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
                Ok(self.pitch_number_to_frequency(num))
            }
        }
    }
    pub fn frequency_ratio(&self, interval: i32) -> f32 {
        (interval as f32 / 12.).exp2()
    }
}


pub fn create_pythagorean_scale() -> Scale {
    let freq_0 = 440. / 27.0_f64;

    Scale::new("pythagorean", freq_0,
        vec![
            ("C", 1.),
            ("Db", 2.0_f64.powi(8) / 3.0_f64.powi(5)),
            ("C#", 3.0_f64.powi(7) / 2.0_f64.powi(11)),
            ("D", 9. / 8.0_f64),
            ("Eb", 2.0_f64.powi(5) / 3.0_f64.powi(3)),
            ("D#", 3.0_f64.powi(9) / 2.0_f64.powi(14)),
            ("E", 81. / 64.0_f64),
            ("F", 4. / 3.0_f64),
            ("Gb", 2.0_f64.powi(10) / 3.0_f64.powi(6)),
            ("F#", 3.0_f64.powi(6) / 2.0_f64.powi(9)),
            ("G", 3. / 2.0_f64),
            ("Ab", 2.0_f64.powi(7) / 3.0_f64.powi(4)),
            ("G#", 3.0_f64.powi(8) / 2.0_f64.powi(12)),
            ("A", 27. / 16.0_f64),
            ("Bb", 16. / 9.0_f64),
            ("A#", 3.0_f64.powi(10) / 2.0_f64.powi(15)),
            ("B", 3.0_f64.powi(5) / 2.0_f64.powi(7)),
        ])
}

pub fn create_natural_scale() -> Scale {
    let freq_0 = (440.0_f64 * 3.) / (5.0_f64 * 16.);

    Scale::new("natural", freq_0,
        vec![
            ("C", 1.),
            ("C#", 25. / 24.0_f64),
            ("D", 9. / 8.0_f64),
            ("D#", 6. / 5.0_f64),
            ("E", 5. / 4.0_f64),
            ("F", 4. / 3.0_f64),
            ("F#", 25. / 18.0_f64),
            ("G", 3. / 2.0_f64),
            ("G#", 25. / 16.0_f64),
            ("A", 5. / 3.0_f64),
            ("A#", 16. / 9.0_f64),
            ("B", 15. / 8.0_f64),
        ])
}

pub fn create_12et_scale() -> Scale {
    let freq_0 = 440.0_f64 / (57. / 12.0_f64).exp2();

    Scale::new("12ET", freq_0,
        vec![
            ("C", 1.),
            ("C#", (1. / 12.0_f64).exp2()),
            ("D", (2. / 12.0_f64).exp2()),
            ("D#", (3. / 12.0_f64).exp2()),
            ("E", (4. / 12.0_f64).exp2()),
            ("F", (5. / 12.0_f64).exp2()),
            ("F#", (6. / 12.0_f64).exp2()),
            ("G", (7. / 12.0_f64).exp2()),
            ("G#", (8. / 12.0_f64).exp2()),
            ("A", (9. / 12.0_f64).exp2()),
            ("A#", (10. / 12.0_f64).exp2()),
            ("B", (11. / 12.0_f64).exp2()),
        ])
}

pub fn create_17et_scale() -> Scale {
    let freq_0 = 440.0_f64 / (81. / 17.0_f64).exp2();

    Scale::new("17ET", freq_0,
        vec![
            ("C", 1.),
            ("Db", (1. / 17.0_f64).exp2()),
            ("C#", (2. / 17.0_f64).exp2()),
            ("D", (3. / 17.0_f64).exp2()),
            ("Eb", (4. / 17.0_f64).exp2()),
            ("D#", (5. / 17.0_f64).exp2()),
            ("E", (6. / 17.0_f64).exp2()),
            ("F", (7. / 17.0_f64).exp2()),
            ("Gb", (8. / 17.0_f64).exp2()),
            ("F#", (9. / 17.0_f64).exp2()),
            ("G", (10. / 17.0_f64).exp2()),
            ("Ab", (11. / 17.0_f64).exp2()),
            ("G#", (12. / 17.0_f64).exp2()),
            ("A", (13. / 17.0_f64).exp2()),
            ("Bb", (14. / 17.0_f64).exp2()),
            ("A#", (15. / 17.0_f64).exp2()),
            ("B", (16. / 17.0_f64).exp2()),
        ])
}

pub fn create_19et_scale() -> Scale {
    let freq_0 = 440.0_f64 / (90. / 19.0_f64).exp2();

    Scale::new("19ET", freq_0,
        vec![
            ("C", 1.),
            ("C#", (1. / 19.0_f64).exp2()),
            ("Db", (2. / 19.0_f64).exp2()),
            ("D", (3. / 19.0_f64).exp2()),
            ("D#", (4. / 19.0_f64).exp2()),
            ("Eb", (5. / 19.0_f64).exp2()),
            ("E", (6. / 19.0_f64).exp2()),
            ("Fb", (7. / 19.0_f64).exp2()),
            ("F", (8. / 19.0_f64).exp2()),
            ("F#", (9. / 19.0_f64).exp2()),
            ("Gb", (10. / 19.0_f64).exp2()),
            ("G", (11. / 19.0_f64).exp2()),
            ("G#", (12. / 19.0_f64).exp2()),
            ("Ab", (13. / 19.0_f64).exp2()),
            ("A", (14. / 19.0_f64).exp2()),
            ("A#", (15. / 19.0_f64).exp2()),
            ("Bb", (16. / 19.0_f64).exp2()),
            ("B", (17. / 19.0_f64).exp2()),
            ("Cb", (18. / 19.0_f64).exp2()),
        ])
}

pub fn create_24et_scale() -> Scale {
    let freq_0 = 440.0_f64 / (114. / 24.0_f64).exp2();

    Scale::new("24ET", freq_0,
        vec![
            ("C", 1.),
            ("Cd", (1. / 24.0_f64).exp2()),
            ("C#", (2. / 24.0_f64).exp2()),
            ("Db", (3. / 24.0_f64).exp2()),
            ("D", (4. / 24.0_f64).exp2()),
            ("Dd", (5. / 24.0_f64).exp2()),
            ("D#", (6. / 24.0_f64).exp2()),
            ("Eb", (7. / 24.0_f64).exp2()),
            ("E", (8. / 24.0_f64).exp2()),
            ("Fb", (9. / 24.0_f64).exp2()),
            ("F", (10. / 24.0_f64).exp2()),
            ("Fd", (11. / 24.0_f64).exp2()),
            ("F#", (12. / 24.0_f64).exp2()),
            ("Gb", (13. / 24.0_f64).exp2()),
            ("G", (14. / 24.0_f64).exp2()),
            ("Gd", (15. / 24.0_f64).exp2()),
            ("G#", (16. / 24.0_f64).exp2()),
            ("Ab", (17. / 24.0_f64).exp2()),
            ("A", (18. / 24.0_f64).exp2()),
            ("Ad", (19. / 24.0_f64).exp2()),
            ("A#", (20. / 24.0_f64).exp2()),
            ("Bb", (21. / 24.0_f64).exp2()),
            ("B", (22. / 24.0_f64).exp2()),
            ("Cb", (23. / 24.0_f64).exp2()),
        ])
}

pub fn create_53et_scale() -> Scale {
    let freq_0 = 440.0_f64 / (252. / 53.0_f64).exp2();

    Scale::new("53ET", freq_0,
        vec![
            ("C", 1.),
            ("^C", (1. / 53.0_f64).exp2()),
            ("^^C", (2. / 53.0_f64).exp2()),
            ("vvC#", (3. / 53.0_f64).exp2()),
            ("vC#", (4. / 53.0_f64).exp2()),
            ("C#", (5. / 53.0_f64).exp2()),
            ("^C#", (6. / 53.0_f64).exp2()),
            ("vvD", (7. / 53.0_f64).exp2()),
            ("vD", (8. / 53.0_f64).exp2()),
            ("D", (9. / 53.0_f64).exp2()),
            ("^D", (10. / 53.0_f64).exp2()),
            ("^^D", (11. / 53.0_f64).exp2()),
            ("vvD#", (12. / 53.0_f64).exp2()),
            ("vD#", (13. / 53.0_f64).exp2()),
            ("D#", (14. / 53.0_f64).exp2()),
            ("^D#", (15. / 53.0_f64).exp2()),
            ("vvE", (16. / 53.0_f64).exp2()),
            ("vE", (17. / 53.0_f64).exp2()),
            ("E", (18. / 53.0_f64).exp2()),
            ("^E", (19. / 53.0_f64).exp2()),
            ("^^E", (20. / 53.0_f64).exp2()),
            ("vF", (21. / 53.0_f64).exp2()),
            ("F", (22. / 53.0_f64).exp2()),
            ("^F", (23. / 53.0_f64).exp2()),
            ("^^F", (24. / 53.0_f64).exp2()),
            ("vvF#", (25. / 53.0_f64).exp2()),
            ("vF#", (26. / 53.0_f64).exp2()),
            ("F#", (27. / 53.0_f64).exp2()),
            ("^F#", (28. / 53.0_f64).exp2()),
            ("vvG", (29. / 53.0_f64).exp2()),
            ("vG", (30. / 53.0_f64).exp2()),
            ("G", (31. / 53.0_f64).exp2()),
            ("^G", (32. / 53.0_f64).exp2()),
            ("^^G", (33. / 53.0_f64).exp2()),
            ("vvG#", (34. / 53.0_f64).exp2()),
            ("vG#", (35. / 53.0_f64).exp2()),
            ("G#", (36. / 53.0_f64).exp2()),
            ("^G#", (37. / 53.0_f64).exp2()),
            ("vvA", (38. / 53.0_f64).exp2()),
            ("vA", (39. / 53.0_f64).exp2()),
            ("A", (40. / 53.0_f64).exp2()),
            ("^A", (41. / 53.0_f64).exp2()),
            ("^^A", (42. / 53.0_f64).exp2()),
            ("vvA#", (43. / 53.0_f64).exp2()),
            ("vA#", (44. / 53.0_f64).exp2()),
            ("A#", (45. / 53.0_f64).exp2()),
            ("^A#", (46. / 53.0_f64).exp2()),
            ("vvB", (47. / 53.0_f64).exp2()),
            ("vB", (48. / 53.0_f64).exp2()),
            ("B", (49. / 53.0_f64).exp2()),
            ("^B", (50. / 53.0_f64).exp2()),
            ("^^B", (51. / 53.0_f64).exp2()),
            ("vC", (52. / 53.0_f64).exp2()),
        ])
}

pub struct Collection {
    map: HashMap<&'static str, Scale>,
}
impl Collection {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        map.insert("pythagorean", create_pythagorean_scale());
        map.insert("natural", create_natural_scale());
        map.insert("12ET", create_12et_scale());
        map.insert("17ET", create_17et_scale());
        map.insert("19ET", create_19et_scale());
        map.insert("53ET", create_53et_scale());

        Self {
            map,
        }
    }

    pub fn values<'a>(&'a self) -> std::collections::hash_map::Values<'_, &str, Scale> {
        self.map.values().into_iter()
    }

    pub fn fetch<'a>(&'a self, scale_name: &str) -> Result<&'a Scale, failure::Error> {
        match self.map.get(scale_name) {
            Some(scale) => Ok(scale),
            None => Err(failure::err_msg(format!("Tseq scale {} unknown!", scale_name))),
        }
    }
}

#[test]
fn test_pitch_name_to_number() {
    let scale = create_53et_scale();
    assert!(scale.pitch_name_to_number("vC0").unwrap() == 105);
}

#[test]
fn test_pitch_number_to_name() {
    let scale = create_53et_scale();
    let name = scale.pitch_number_to_name(105);
    assert_eq!(&name, "vC0");
}
