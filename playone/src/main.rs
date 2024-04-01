#![allow(dead_code, unused_variables, unused_imports)]
extern crate failure;
extern crate session;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;

use session::player::Player;
use session::state::State;

fn main() {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        let filename = &args[i];

        match play(filename) {
            Ok(_) => {}
            e => {
                eprintln!("playing {} failed : {:?}", filename, e);
            }
        }
    }
}

fn play(filename: &str) -> Result<(), failure::Error> {
    let band_description = String::from_utf8(fs::read(filename)?)?;
    let mut player = Player::new(band_description)?;

    let mut state = player.play()?;

    while state != State::Exited {
        state = player.wait()?;
        println!("Player state : {}", state.to_string());
    }
    Ok(())
}
