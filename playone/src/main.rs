#![allow(dead_code, unused_variables, unused_imports)]
extern crate failure;
extern crate session;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use session::player::Player;
use session::state::State;

fn main() {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        let file_path = PathBuf::from(&args[i]);

        match play(&file_path) {
            Ok(_) => {}
            e => {
                eprintln!("playing {} failed : {:?}", file_path.display(), e);
            }
        }
    }
}

fn play(file_path: &Path) -> Result<(), failure::Error> {
    let band_description = String::from_utf8(fs::read(file_path)?)?;
    let session_folder = file_path.parent().unwrap();
    let mut player = Player::new(&band_description, session_folder)?;

    let mut state = player.play()?;

    while state != State::Exited {
        state = player.wait()?;
        println!("Player state : {}", state.to_string());
    }
    Ok(())
}
