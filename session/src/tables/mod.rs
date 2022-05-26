pub mod sinramp;

#[cfg(test)]
mod tests {
    use std::f32;
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
}
