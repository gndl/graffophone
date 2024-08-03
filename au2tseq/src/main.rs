#![allow(dead_code, unused_variables, unused_imports)]
extern crate failure;
extern crate rustfft;

extern crate audiofile;
extern crate scale;

use std::env;
/*
use std::fs;
use std::fs::File;
use std::io::Read;
use std::f32;
*/
use std::f64::consts::PI;

use rustfft::{FftPlanner, num_complex::Complex};

use audiofile::reader::Reader;
use scale::pitch_fetcher;

const SAMPLE_RATE: f64 = 44100.;
const FREQUENCY_STEP: f64 = 6.;
const CHUNK_SIZE: usize = (SAMPLE_RATE / FREQUENCY_STEP) as usize;
const CHUNK_DURATION: f64 = 1.0 / FREQUENCY_STEP;
const PEAK_THRESHOLD: f64 = 24000.;// * 1200.;

fn chunk_freq(chunk: &Vec<Complex<f64>>) -> f32 {
    let mut a_max = 0.0f64;
    let half_chunk_size = CHUNK_SIZE / 2;

    let mut freqs = Vec::with_capacity(half_chunk_size);

    for i in 0..(half_chunk_size) {
        freqs.push(chunk[i].re * chunk[i].re + chunk[i].im * chunk[i].im);
    }

    let mut freqs_sum = 0.;

    for i in 0..(half_chunk_size) {
        freqs_sum += freqs[i];
    }

    let peak_threshold = 250. * freqs_sum / half_chunk_size as f64;


    for i in 0..(half_chunk_size) {
        // let a = chunk[i].im * chunk[CHUNK_SIZE - i - 1].im;
        // let a_m = chunk[i].re * chunk[i].re + chunk[i].im * chunk[i].im;
        let a_m = freqs[i];
        // if a < (-1. * PEAK_THRESHOLD * PEAK_THRESHOLD) {
        a_max = a_max.max(a_m);

        // println!("{} {}Hz : re = {}, im = {} => {}", i, (i as f64 * SAMPLE_RATE) / CHUNK_SIZE as f64, chunk[i].re, chunk[i].im, a_m);

        if a_m > peak_threshold {
            let f_m = (i as f64 * SAMPLE_RATE) / CHUNK_SIZE as f64;
            // let a_n = chunk[i + 1].re * chunk[i + 1].re + chunk[i + 1].im * chunk[i + 1].im;
            let a_n = freqs[i + 1];
            
            if a_n < peak_threshold {
                // println!("{} : re = {}, im = {}, a = {} => {}Hz\n", i, chunk[i].re, chunk[i].im, a_m, f_m);
                return f_m as f32;
            }
            else {
                let f_n = ((i + 1) as f64 * SAMPLE_RATE) / CHUNK_SIZE as f64;
                // println!("{} : re = {}, im = {}, a = {} => {}Hz", i, chunk[i].re, chunk[i].im, a_m, f_m);
                // println!("{} : re = {}, im = {}, a = {} => {}Hz", i + 1, chunk[i + 1].re, chunk[i + 1].im, a_n, f_n);
                // let f = (f_m * a_m + f_n * a_n) / (a_m + a_n);
                let f = (f_m + f_n) / 2.;
                // println!(" => {}Hz\n", f);
                return f as f32;
            }
        }
    }
    // println!("a max : {}", a_max);
    0.
}

fn file_freqs(filename: &str) -> Vec<f32> {
    let mut file_reader = Reader::new(filename, SAMPLE_RATE as usize).expect("Invalid audio file");
    let mut channels = Vec::new();

    for _ in 0..file_reader.channels() {
        channels.push(vec![0.; CHUNK_SIZE]);
    }

    let mut freqs = Vec::new();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(CHUNK_SIZE);

    let mut chunk = Vec::with_capacity(CHUNK_SIZE);
    let mut scratch = vec![Complex{ re: 0.0f64, im: 0.0f64 }; CHUNK_SIZE];

    while file_reader.read_samples(&mut channels, CHUNK_SIZE).expect("Invalid audio file") == CHUNK_SIZE {

        for t in 0..CHUNK_SIZE {
            let re = channels[0][t] as f64;
            chunk.push(Complex{ re, im: 0.0f64});
        }

        fft.process_with_scratch(&mut chunk, &mut scratch);
        let freq = chunk_freq(&chunk);
        freqs.push(freq);
        chunk.clear();
    }
    freqs
}

fn file_freqs_and_durations(filename: &str) -> Vec<(f32, f64)> {
    let mut freqs = Vec::new();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(CHUNK_SIZE);

    let mut chunk = vec![Complex{ re: 0.0f64, im: 0.0f64 }; CHUNK_SIZE];
    let mut scratch = vec![Complex{ re: 0.0f64, im: 0.0f64 }; CHUNK_SIZE];

    let buf;
    {
        let mut file_reader = Reader::new(filename, SAMPLE_RATE as usize).expect("Invalid audio file");
        let mut channels = file_reader.read_all_samples().expect("Invalid audio file");
        buf = channels.swap_remove(0);
    }
    let step = SAMPLE_RATE as usize / 100;
    let mut prev_freq = 0.;
    let mut freq_start = 0;
    let mut pos = 0;
    let mut first_freq_step = true;
    
    while pos < buf.len() - CHUNK_SIZE {
        for t in 0..CHUNK_SIZE {
            chunk[t].re = buf[pos + t] as f64;
            chunk[t].im = 0.0f64;
        }
        fft.process_with_scratch(&mut chunk, &mut scratch);
        let freq = chunk_freq(&chunk);

        if freq != prev_freq && !first_freq_step {
            let samples = pos - freq_start + CHUNK_SIZE - step;
            freqs.push((prev_freq, samples as f64 / SAMPLE_RATE));
            
            freq_start += samples;
            pos = freq_start;
            first_freq_step = true;
        }
        else {
            pos += step;
            first_freq_step = false;
        }
        prev_freq = freq;
    }
    freqs
}


pub struct Note {
    pub pitch: String,
    pub duration: f64,
    pub duration_on: f64,
}

fn freqs_notes(freqs: &Vec<f32>) -> Vec<Note> {
    let mut notes = Vec::new();

    if !freqs.is_empty() {
        let mut freq = freqs[0];
        let mut duration_count = 0.;
        let mut duration_on_count = 0.;
        let mut time_off = false;

        let pitch_fetchers = pitch_fetcher::Collection::new();
        let pitch_fetcher = pitch_fetchers.default();

        for f in freqs {
            if *f > 8. {
                if *f != freq || time_off {
                    if freq > 8. {
                        let pitch = pitch_fetcher.fetch_pitch(freq).expect("No pitch for freq");

                        notes.push(Note{pitch, duration: duration_count * CHUNK_DURATION, duration_on: duration_on_count * CHUNK_DURATION});
                    }

                    freq = *f;
                    duration_count = 0.;
                    duration_on_count = 0.;
                }

                duration_on_count += 1.;
                time_off = false;
            }
            else {
                time_off = true;
            }
            duration_count += 1.;
        }
        if freq > 8. {
            let pitch = pitch_fetcher.fetch_pitch(freq).expect("No pitch for freq");

            notes.push(Note{pitch, duration: duration_count * CHUNK_DURATION, duration_on: duration_on_count * CHUNK_DURATION});
        }
    }
    notes
}


fn freqs_and_durations_notes(freqs_and_durations: &Vec<(f32, f64)>) -> Vec<Note> {
    let mut notes = Vec::new();

    if !freqs_and_durations.is_empty() {
        let (mut freq, mut duration_on)  = freqs_and_durations[0];
        let mut duration_off = 0.;

        let pitch_fetchers = pitch_fetcher::Collection::new();
        let pitch_fetcher = pitch_fetchers.default();

        for (f, dur) in freqs_and_durations {
            if *f > 16. {
                if freq > 16. {
                    let pitch = pitch_fetcher.fetch_pitch(freq).expect("No pitch for freq");

                    notes.push(Note{pitch, duration: duration_on + duration_off, duration_on});
                }

                freq = *f;
                duration_on = *dur;
                duration_off = 0.;
            }
            else {
                duration_off += *dur;
            }
        }
        if freq > 16. {
            let pitch = pitch_fetcher.fetch_pitch(freq).expect("No pitch for freq");

            notes.push(Note{pitch, duration: duration_on + duration_off, duration_on});
        }
    }
    notes
}


pub struct Hit {
    pub time: f64,
    pub on_rate: f64,
}

fn notes_hits(notes: &Vec<Note>) -> (f64, Vec<Hit>, f64) {
    let mut min_duration = f64::MAX;

    for note in notes {
        if note.duration < min_duration {
            min_duration = note.duration;
        }
    }

    let mut time = 0.;
    let mut hits = Vec::new();

    for note in notes {
        let on_rate = (((note.duration_on / note.duration) * 10.).round()) / 10.;

        hits.push(Hit{time, on_rate});

        time = (((time + note.duration / min_duration) * 10.).round()) / 10.;
    }

    let bpm = 60. / min_duration;

    (bpm, hits, time)
}

fn print_tseq(seqname: &str, bpm: f64, notes: &Vec<Note>, hits: &Vec<Hit>, times: f64) {
    print!("pitchs {} :", seqname);
    for note in notes {
        print!(" {}", note.pitch);
    }
    println!("");
    println!("beat {} : {}", seqname, bpm);

    print!("hits {} :", seqname);
    for hit in hits {
        print!(" {}", hit.time);
    }
    println!(" % {}", times);

    print!("durations {} :", seqname);
    for hit in hits {
        print!(" {}", hit.on_rate);
    }
    println!("");

    println!("seq {} : ?beat={} {}&{}-{}", seqname, bpm, seqname, seqname, seqname);
    println!("seqout {} : @{}", seqname, seqname);
    println!("");
}

fn main() {
    let _ = audiofile::init();
    let args: Vec<String> = env::args().collect();
   let filename = &args[1];
//    println!("filename {} :", filename);
   let seqname = "riff";

   let freqs = file_freqs(filename);
    let notes = freqs_notes(&freqs);
    let (bpm, hits, times) = notes_hits(&notes);
    print_tseq("thick", bpm, &notes, &hits, times);

    let freqs_and_durations = file_freqs_and_durations(filename);
    let notes = freqs_and_durations_notes(&freqs_and_durations);
    let (bpm, hits, times) = notes_hits(&notes);
    print_tseq("thin", bpm, &notes, &hits, times);
}

fn test_file_freqs(_: &str) -> Vec<f32> {
    let mut freqs = Vec::new();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(CHUNK_SIZE);

    let mut chunk = Vec::with_capacity(CHUNK_SIZE);
    let mut scratch = vec![Complex{ re: 0.0f64, im: 0.0f64 }; CHUNK_SIZE];

    let basic_freq: f64 = 55.;
    let rs = [1., 1.5, 2.5, 3.1, 3.5, 5.5, 12., 100., 2., 3., 4., 8., 16.];
    // let rs = [2., 3.];

    let fs = rs.map(|r| r * basic_freq);
    let pitch_fetchers = pitch_fetcher::Collection::new();
    let pitch_fetcher = pitch_fetchers.default();

    print!("pitchs src :");
    for f in fs {
        let pitch = pitch_fetcher.fetch_pitch(f as f32).expect("No pitch for freq");
        print!(" {}", pitch);
    }
    println!("");

    let c = (PI * 2. * basic_freq) / SAMPLE_RATE;

    let cs = rs.map(|r| r * c);

    for c in cs {
        for _ in 0..6 {
            for t in 0..CHUNK_SIZE {
                chunk.push(Complex{ re: (t as f64 * c).sin(), im: 0.0f64 });
            }
            fft.process_with_scratch(&mut chunk, &mut scratch);
            // freqs.push(chunk_positive_freq(&chunk));
            // freqs.push(chunk_negative_freq(&chunk));
            freqs.push(chunk_freq(&chunk));
            chunk.clear();
        }
    }
    /*
    */
    for _ in 0..6 {
        for t in 0..CHUNK_SIZE {
            chunk.push(Complex{ re: 
                (t as f64 * c).sin() + 
                (t as f64 * c * 1.5).sin() + 
                (t as f64 * c * 2.5).sin() + 
                (t as f64 * c * 3.5).sin() + 
                (t as f64 * c * 5.5).sin() +
                (t as f64 * c * 12.).sin() +
                (t as f64 * c * 100.).sin(), 
                im: 0.0f64
                }
            );
        }

        fft.process_with_scratch(&mut chunk, &mut scratch);
        let freq = chunk_freq(&chunk);
        freqs.push(freq);
        chunk.clear();
    }
    freqs
}
/*
fn chunk_positive_freq(chunk: &Vec<Complex<f64>>) -> f32 {
    
for i in 0..CHUNK_SIZE {
    if chunk[CHUNK_SIZE - i - 1].im > PEAK_THRESHOLD {
        return ((i + 1) as f32 * SAMPLE_RATE) / CHUNK_SIZE as f32;
    }
}
0.
}

fn chunk_negative_freq(chunk: &Vec<Complex<f64>>) -> f32 {
    
for i in 1..(CHUNK_SIZE / 10) {
    println!("{} {}Hz : re = {}, im = {}", i, (i as f32 * SAMPLE_RATE) / CHUNK_SIZE as f32, chunk[i].re, chunk[i].im);
    
    if chunk[i].im < (-1. * PEAK_THRESHOLD) {
        return (i as f32 * SAMPLE_RATE) / CHUNK_SIZE as f32;
    }
}
0.
}
*/
