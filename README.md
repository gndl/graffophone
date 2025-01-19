# Graffophone

Modular audio processing application including sequencer, synthesis and mixing functions.

Supported formats :
- Audio, MIDI
- Scales : 12 ET, 17 ET, 19 ET, 53 ET, natural, pythagorean
- Plugins : Lv2
- Sample rates (Hz) : 8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000 
- Codecs : FLAC, MP3, Ogg Vorbis, Opus, WAV


![Graffophone](https://github.com/gndl/graffophone/wiki/graffophone-0.2.0.png)

Building and installing Graffophone
==============================


Configuration
-------------

Prerequisites: rust >= 1.73.0, ffmpeg 7, liblilv-dev >= 0.24

Compilation
-----------

    $ cargo build --bin graffophone --release


Execution
---------

    $ cargo run --bin graffophone --release


