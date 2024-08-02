extern crate audiofile;

use talker::ctalker;
use talker::audio_format::AudioFormat;
use talker::data::Data;
use talker::talker::{CTalker, Talker, TalkerBase};
use talker::talker_handler::TalkerHandlerBase;
use audiofile::reader::Reader;

use crate::channel;

pub const MODEL: &str = "AudioFile";

pub struct AudioFileInput {
    channels: Vec<Vec<f32>>,
}

impl AudioFileInput {
    pub fn new(base: TalkerBase) -> Result<CTalker, failure::Error> {
        base.set_data(Data::File("Click here to select a file".to_string()));

        Ok(ctalker!(
            base,
            Self {
                channels: Vec::new(),
            }
        ))
    }

    pub fn descriptor() -> TalkerHandlerBase {
        TalkerHandlerBase::builtin("Generator", MODEL, MODEL)
    }
}

impl Talker for AudioFileInput {
    fn set_data_update(
        &mut self,
        base: &TalkerBase,
        data: Data,
    ) -> Result<Option<TalkerBase>, failure::Error> {
        match data {
            Data::File(ref filename) => {
                let mut new_base = base.with(None, None, None);

                let sample_rate = AudioFormat::sample_rate();
                let mut file_reader = Reader::new(filename, sample_rate)?;

                let channels_count = file_reader.channels();
                let channels_names = channel::Layout::channels_names_from_channels(channels_count);

                for c in 0..channels_count {
                    let tag = if c < channels_names.len() {
                        Some(channels_names[c])
                    }
                    else {
                        None
                    };
                    new_base.add_audio_voice(tag, 0.);
                }

                if base.is_effective() {
                    self.channels =file_reader.read_all_samples()?;
                }
                    
                new_base.set_data(data);
                Ok(Some(new_base))
            }
            _ => Err(failure::err_msg(format!("{} data type {} is not File", MODEL, data.type_str()))),
        }
    }

    fn talk(&mut self, base: &TalkerBase, port: usize, tick: i64, len: usize) -> usize {

        if port < self.channels.len() {
            let channel = &self.channels[port];
            let voice_buf = base.voice(port).audio_buffer();
            let t = tick as usize;
            let channel_len = channel.len();

            let ln = if t < channel_len {
                len.min(channel_len - t)
            }
            else {
                0
            };

            for i in 0..ln {
                voice_buf[i] = channel[t + i];
            }
            for i in ln..len {
                voice_buf[i] = 0.;
            }
            len
        }
        else {
            0
        }
    }
}
