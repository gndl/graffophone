use std::f32;
use std::f64;
use std::f64::consts::PI;

pub mod parabolic;
pub mod round;
pub mod sinramp;
pub mod roundramp;
pub mod earlyramp;
pub mod lateramp;

pub fn fade_len(sample_rate: usize) -> usize {
    sample_rate / 100
}

pub fn create_fadein_fadeout(sample_rate: usize) -> (Vec<f32>, Vec<f32>) {
    let len = fade_len(sample_rate);
    let mut fadein_tab = Vec::with_capacity(len);
    let mut fadeout_tab = Vec::with_capacity(len);

    for i in 0..len {
        let a = ((i as f64 * PI) / (len as f64)) + (PI * 0.5);
        let v = (a.sin() + 1.) * 0.5;
        fadein_tab.push((1. - v) as f32);
        fadeout_tab.push(v as f32);
    }
    (fadein_tab, fadeout_tab)
}

pub fn create_fadeout(sample_rate: usize) -> Vec<f32> {
    let len = fade_len(sample_rate);
    let mut fadeout_tab = Vec::with_capacity(len);

    for i in 0..len {
        let a = ((i as f64 * PI) / (len as f64)) + (PI * 0.5);
        let v = (a.sin() + 1.) * 0.5;
        fadeout_tab.push(v as f32);
    }
    fadeout_tab
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tables::f32;
    use tables::f64;
    use tables::PI;

const RAMP_LEN : usize = 24000;

    #[test]
    fn test_conv() {
        assert!(3.9999_f32 as usize == 3);
        assert!(4.00001 as usize == 4);
    }

    #[test]
    fn create_round() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/round.rs")?;
        let len = 96000;

        writeln!(f, "pub const LEN:usize = {};", len)?;
        writeln!(f, "pub const TAB: [f32; LEN + 1] = [")?;

        let mid_len = len / 2;
        let step = 2. / mid_len as f64;
        let mut neg_part: Vec<f32> = Vec::with_capacity(mid_len);
        let mut x: f64 = -1.;

        for _ in 0..mid_len {
            let y = ((1. - (x * x)).sqrt()) as f32;
            writeln!(f, "{:.10},", y)?;
            neg_part.push(-y);
            x += step;
        }

        for y in neg_part {
            writeln!(f, "{:.10},", y)?;
        }
        writeln!(f, "0.0];")?;
        Ok(())
    }

    #[test]
    fn create_parabolic() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/parabolic.rs")?;
        let len = 96000;

        writeln!(f, "pub const LEN:usize = {};", len)?;
        writeln!(f, "pub const TAB: [f32; LEN + 1] = [")?;

        let mid_len = len / 2;
        let step = 2. / mid_len as f64;
        let mut neg_part: Vec<f32> = Vec::with_capacity(mid_len);
        let mut x: f64 = -1.;

        for _ in 0..mid_len {
            let y = (1. - (x * x)) as f32;
            writeln!(f, "{:.10},", y)?;
            neg_part.push(-y);
            x += step;
        }

        for y in neg_part {
            writeln!(f, "{:.10},", y)?;
        }
        writeln!(f, "0.0];")?;
        Ok(())
    }

    #[test]
    fn create_sinramp() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/sinramp.rs")?;

        writeln!(f, "pub const LEN:usize = {};", RAMP_LEN)?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

        for i in 0..RAMP_LEN {
            let a = ((i as f64 * PI) / (RAMP_LEN as f64)) - (PI * 0.5);
            let v = (a.sin() + 1.) * 0.5;
            writeln!(f, "{:.8},", v as f32)?;
        }
        writeln!(f, "];")?;

        Ok(())
    }

    #[test]
    fn create_roundramp() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/roundramp.rs")?;

        writeln!(f, "pub const LEN:usize = {};", RAMP_LEN)?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

        let mid_len = RAMP_LEN / 2;
        let step = 1. / mid_len as f64;
        let mut x: f64 = 0.;

        for _ in 0..mid_len {
            let y = ((1.-((1. - (x * x)).sqrt())) * 0.5) as f32;
            writeln!(f, "{:.8},", y)?;
            x += step;
        }

        x = -1.;
        for _ in 0..mid_len {
            let y = ((0.5 * ((1. - (x * x)).sqrt())) + 0.5) as f32;
            writeln!(f, "{:.8},", y)?;
            x += step;
        }
        writeln!(f, "];")?;

        Ok(())
    }

    #[test]
    fn create_earlyramp() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/earlyramp.rs")?;

        writeln!(f, "pub const LEN:usize = {};", RAMP_LEN)?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

        let step = 1. / RAMP_LEN as f64;
        let mut x: f64 = 1.;

        for _ in 0..RAMP_LEN {
            let y = (1.-(x * x * x)) as f32;
            writeln!(f, "{:.8},", y)?;
            x -= step;
        }
        writeln!(f, "];")?;

        Ok(())
    }

    #[test]
    fn create_lateramp() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/lateramp.rs")?;

        writeln!(f, "pub const LEN:usize = {};", RAMP_LEN)?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

        let step = 1. / RAMP_LEN as f64;
        let mut x: f64 = 0.;

        for _ in 0..RAMP_LEN {
            let y = (x * x * x) as f32;
            writeln!(f, "{:.8},", y)?;
            x += step;
        }
        writeln!(f, "];")?;

        Ok(())
    }

    #[test]
    fn create_fading() -> Result<(), failure::Error> {
        let len = 1200;
        let mut f_fadein = File::create("src/tables/fadein.rs")?;
        let mut f_fadeout = File::create("src/tables/fadeout.rs")?;

        writeln!(f_fadein, "pub const LEN:usize = {};\npub const TAB: [f32; LEN] = [", len)?;
        writeln!(f_fadeout, "pub const LEN:usize = {};\npub const TAB: [f32; LEN] = [", len)?;

        for i in 0..len {
            let a = ((i as f64 * PI) / (len as f64)) - (PI * 0.5);
            let v = (a.sin() + 1.) * 0.5;
            writeln!(f_fadein, "{:.10},", v as f32)?;
            writeln!(f_fadeout, "{:.10},", (1. - v) as f32)?;
        }
        writeln!(f_fadein, "];")?;
        writeln!(f_fadeout, "];")?;
        Ok(())
    }
}
