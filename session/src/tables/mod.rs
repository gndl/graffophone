pub mod round;
pub mod sinramp;

#[cfg(test)]
mod tests {
    use std::f32;
    use std::f64;
    use std::f64::consts::PI;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn create_sinramp() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/sinramp.rs")?;
        let len = 32768;

        writeln!(f, "pub const LEN:usize = {};", len)?;
        //        writeln!(f, "pub fn get(i: usize) -> f32 {{ let tab:[f32] = [")?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

        for i in 0..len {
            let a = ((i as f64 * PI) / (len as f64)) - (PI * 0.5);
            let v = (a.sin() + 1.) * 0.5;
            writeln!(f, "{:.10},", v as f32)?;
        }
        writeln!(f, "];")?;
        //        writeln!(f, "]; tab[i]}}")?;
        Ok(())
    }

    #[test]
    fn create_round() -> Result<(), failure::Error> {
        let mut f = File::create("src/tables/round.rs")?;
        let len = 96000;

        writeln!(f, "pub const LEN:usize = {};", len)?;
        writeln!(f, "pub const TAB: [f32; LEN] = [")?;

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
        writeln!(f, "];")?;
        Ok(())
    }
}
