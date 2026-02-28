# Graffophone

Modular audio processing application including sequencer, synthesis and mixing functions.

Supported formats :
- Audio, MIDI
- Scales : 12 ET, 17 ET, 19 ET, 53 ET, natural, pythagorean
- Plugins : Lv2
- Sample rates (Hz) : 8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000
- Channel layouts : mono, stereo, 2.1, 3.0, 3.0(back), 4.0, quad, quad(side), 3.1, 5.0, 5.0(side), 4.1, 5.1, 5.1(side), 6.0, 6.0(front), hexagonal, 6.1, 6.1(back), 6.1(front), 7.0, .0(front), 7.1, 7.1(wide), 7.1(wide-side), 7.1(top), octagonal, cube, hexadecagonal, 22.2
- Codecs : FLAC, MP3, Ogg Vorbis, Opus, WAV


![Graffophone](https://github.com/gndl/graffophone/wiki/graffophone-0.5.0.png)

## Building and installing Graffophone


### Configuration

Prerequisites: rust >= 1.73.0, ffmpeg 7, liblilv-dev >= 0.24

### Compilation

    $ cargo build --bin graffophone --release


### Execution

    $ cargo run --bin graffophone --release


## Credits

This project is only made possible due to many libraries and tools, including but not limited to:

- [cpal](https://github.com/RustAudio/cpal/)
- [FFmpeg](https://ffmpeg.org/)
- [ffmpeg-next](https://github.com/zmwangx/rust-ffmpeg/)
- [GTK](https://www.gtk.org/)
- [gtk-rs](https://gtk-rs.org/)
- [Lv2](https://lv2plug.in/)
- [livi-rs](https://github.com/wmedrano/livi-rs/)
- [nom](https://github.com/rust-bakery/nom/)
- [Rust](https://rust-lang.org/)
