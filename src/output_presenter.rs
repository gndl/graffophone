
use talker::identifier::{Id, Identifier};
use session::{channel, output::ROutput};

pub const DEFAULT_CODEC: &str = "flac";
pub const CODECS_LABELS: [&str; 6] = ["FLAC", "MP3", "Ogg Vorbis", "Opus", "WAV 16-bit", "WAV 24-bit"];
pub const CODECS_NAMES: [&str; 6] = [DEFAULT_CODEC, "libmp3lame", "libvorbis", "libopus", "pcm_s16le", "pcm_s24le"];
pub const CODEC_CONTAINERS_EXTENTIONS: [&str; 6] = [DEFAULT_CODEC, "mp3", "ogg", "opus", "wav", "wav"];

pub const DEFAULT_SAMPLE_RATE: usize = 44100;
pub const SAMPLE_RATES: [&str; 9] = ["8000", "11025", "16000", "22050", "32000", "44100", "48000", "88200", "96000"];

pub const DEFAULT_AUDIO_FILE_EXTENTION: &str = DEFAULT_CODEC;

pub struct OutputPresenter {
    identifier: Identifier,
    codec_name: String,
    sample_rate: usize,
    channel_layout: String,
    file_path: String,
}

impl OutputPresenter {
    pub fn new(
        identifier: Identifier,
        codec_name: &str,
        sample_rate: usize,
        channel_layout: &str,
        file_path: &str,
    ) -> OutputPresenter {
        Self {
            identifier,
            codec_name: codec_name.to_string(),
            sample_rate,
            channel_layout: channel_layout.to_string(),
            file_path: file_path.to_string(),
        }
    }
            
    pub fn from(output: &ROutput) -> OutputPresenter {
        let out = output.borrow();
        let identifier = out.identifier().borrow().clone();
        let codec_name = out.codec_name();
        let sample_rate = out.sample_rate();
        let channel_layout = out.channel_layout();
        let file_path = out.file_path();
        OutputPresenter::new(identifier, codec_name, sample_rate, channel_layout, file_path)
    }

    pub fn identifier<'a>(&'a self) -> &'a Identifier {
        &self.identifier
    }
    pub fn id(&self) -> Id {
        self.identifier.id()
    }

    pub fn codec_name<'a>(&'a self) -> &'a str {
        self.codec_name.as_str()
    }

    pub fn set_codec_name(&mut self, value: &str) {
        self.codec_name = value.to_string();
    }

    pub fn codec_index(&self) -> usize {
        let codec = self.codec_name();

        for idx in 0..CODECS_NAMES.len() {
            if CODECS_NAMES[idx] == codec {
                return idx;
            }
        }
        eprintln!("Unknow sample format {}. Fallback to {}.", codec, CODECS_NAMES[0]);
        0
    }

    pub fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, value: usize) {
        self.sample_rate = value;
    }

    pub fn sample_rate_index(&self) -> usize {
        let sample_rate = self.sample_rate().to_string();

        for idx in 0..SAMPLE_RATES.len() {
            if SAMPLE_RATES[idx] == sample_rate {
                return idx;
            }
        }
        eprintln!("Unknow sample rate {}. Fallback to {}.", sample_rate, SAMPLE_RATES[0]);
        0
    }

    pub fn channel_layout<'a>(&'a self) -> &'a str {
        self.channel_layout.as_str()
    }

    pub fn set_channel_layout(&mut self, value: &str) {
        self.channel_layout = value.to_string();
    }

    pub fn channel_layout_index(&self) -> usize {
        channel::Layout::index(self.channel_layout())
    }

    pub fn file_path<'a>(&'a self) -> &'a str {
        self.file_path.as_str()
    }

    pub fn set_file_path(&mut self, value: &str) {
        self.file_path = value.to_string();
    }
}