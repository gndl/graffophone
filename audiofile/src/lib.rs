extern crate failure;
extern crate ffmpeg_next as ffmpeg;

pub mod reader;
pub mod writer;


pub fn init() -> Result<(), failure::Error> {
    ffmpeg::init().map_err(|e| failure::err_msg(format!("FFMPEG init error : {}", e)))
}
