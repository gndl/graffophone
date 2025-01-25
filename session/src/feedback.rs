use std::cell::RefCell;
use std::rc::Rc;

use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf;
use ringbuf::RingBuffer;
use talker::audio_format::AudioFormat;

use talker::identifier::RIdentifier;

use tables::fadeout;
use crate::audio_data::Vector;
use crate::{channel, output};
use crate::output::{Output, ROutput};

pub const MODEL: &str = "feedback";

pub type AudioProducer = ringbuf::Producer<f32>;

pub struct AudioStream {
    stream: cpal::Stream,
    producer: AudioProducer,
}
pub struct Feedback {
    identifier: RIdentifier,
    sample_rate: usize,
    nb_samples: usize,
    nb_channels: usize,
    audio_stream: Option<AudioStream>,
}

impl Feedback {
    pub fn new(nb_samples: usize) -> Result<Feedback, failure::Error> {
        // Default devices.
        let output_device = cpal::default_host()
            .default_output_device()
            .expect("failed to get default output device");

        println!("Using default output device: \"{}\"", output_device.name()?);

        let config: cpal::StreamConfig = output_device.default_output_config()?.into();

        Ok(Self {
            identifier: output::new_identifier("", MODEL),
            sample_rate: AudioFormat::sample_rate(),
            nb_samples,
            nb_channels: config.channels as usize,
            audio_stream: None,
        })
    }

    pub fn new_ref(nb_samples: usize) -> Result<ROutput, failure::Error> {
        Ok(Rc::new(RefCell::new(Feedback::new(nb_samples)?)))
    }

    fn make_audio_stream(
        nb_channels: usize,
        nb_samples: usize,
    ) -> Result<AudioStream, failure::Error> {
        let output_device = cpal::default_host()
            .default_output_device()
            .expect("failed to get default output device");

        let mut config: cpal::StreamConfig = output_device.default_output_config()?.into();
        config.sample_rate = cpal::SampleRate(AudioFormat::sample_rate() as u32);

        let latency_samples = nb_samples * nb_channels as usize;

        // The buffer to share samples
        let ring = RingBuffer::new(latency_samples * 5);
        let (producer, mut consumer) = ring.split();

        let output_data_fn = move |data: &mut [f32], _: &_| {
            for sample in data {
                let mut ov = consumer.pop();

                while ov == None {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    ov = consumer.pop();
                }
                *sample = ov.unwrap_or(0.0);
            }
        };

        let err_fn = |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        let stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None)?;

        stream
            .play()
            .map_err(|e| failure::err_msg(format!("Feedback::open error : {}", e)))?;

        Ok(AudioStream { producer, stream })
    }

    pub fn fade_len(&self) -> usize {
        fadeout::LEN
    }
    
    pub fn write_fadein(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        let in_chan_end = channels.len() - 1;
        let mut in_chan_idx = 0;

        let last = fadeout::LEN - 1;
        let fade_len = fadeout::LEN.min(nb_samples_per_channel);

        let mut channels_buffers = Vec::with_capacity(self.nb_channels);

        for _ in 0..self.nb_channels {
            let in_chan = &channels[in_chan_idx];
            let mut out_chan = Vec::with_capacity(nb_samples_per_channel);

            for i in 0..fade_len {
                out_chan.push(in_chan[i] * fadeout::TAB[last - i]);
            }
            for i in fade_len..nb_samples_per_channel {
                out_chan.push(in_chan[i]);
            }
            channels_buffers.push(out_chan);

            if in_chan_idx < in_chan_end {
                in_chan_idx += 1;
            }
        }
        self.write(&channels_buffers, nb_samples_per_channel)
    }

    pub fn write_fadeout(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        let in_chan_end = channels.len() - 1;
        let mut in_chan_idx = 0;

        let fade_start = if fadeout::LEN < nb_samples_per_channel {
            nb_samples_per_channel - fadeout::LEN
        }
        else {
            0
        };

        let mut channels_buffers = Vec::with_capacity(self.nb_channels);

        for _ in 0..self.nb_channels {
            let in_chan = &channels[in_chan_idx];
            let mut out_chan = Vec::with_capacity(nb_samples_per_channel);

            for i in 0..fade_start {
                out_chan.push(in_chan[i]);
            }
            for i in fade_start..nb_samples_per_channel {
                out_chan.push(in_chan[i] * fadeout::TAB[i - fade_start]);
            }
            channels_buffers.push(out_chan);

            if in_chan_idx < in_chan_end {
                in_chan_idx += 1;
            }
        }
        self.write(&channels_buffers, nb_samples_per_channel)
    }
}

impl Output for Feedback {
    fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }

    fn model(&self) -> String{
        MODEL.to_string()
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn channel_layout<'a>(&'a self) -> &'a str{
        channel::Layout::from_channels(self.nb_channels)
    }

    fn channels(&self) -> usize {
        self.nb_channels
    }

    fn channels_names(&self) -> Vec<&'static str> {
        channel::Layout::channels_names_from_channels(self.nb_channels)
    }

    fn open(&mut self) -> Result<(), failure::Error> {
        println!("Feedback::open");
        let audio_stream = Feedback::make_audio_stream(self.nb_channels, self.nb_samples)?;

        self.audio_stream = Some(audio_stream);
        Ok(())
    }

    fn write(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        match self.audio_stream.iter_mut().next() {
            Some(audio_stream) => {
                let mut output_fell_behind = 0;
                let in_chan_end = channels.len() - 1;

                for i in 0..nb_samples_per_channel {
                    let mut in_chan_idx = 0;

                    for _ in 0..self.nb_channels {
                        let sample = channels[in_chan_idx][i];

                        while audio_stream.producer.push(sample).is_err() && output_fell_behind < 30
                        {
                            std::thread::sleep(std::time::Duration::from_millis(20));
                            output_fell_behind = output_fell_behind + 1;
                        }

                        if in_chan_idx < in_chan_end {
                            in_chan_idx += 1;
                        }
                    }
                }
                if output_fell_behind == 30 {
                    eprintln!("output stream fell behind: try increasing latency");
                }
                Ok(())
            }
            None => Err(failure::err_msg(
                "Feedback::write error : no open audio stream",
            )),
        }
    }

    fn pause(&mut self) -> Result<(), failure::Error> {
        match self.audio_stream.iter_mut().next() {
            Some(audio_stream) => audio_stream
                .stream
                .pause()
                .map_err(|e| failure::err_msg(format!("Feedback::pause error : {}", e))),
            None => Err(failure::err_msg(
                "Feedback::pause error : no open audio stream",
            )),
        }
    }

    fn run(&mut self) -> Result<(), failure::Error> {
        match self.audio_stream.iter_mut().next() {
            Some(audio_stream) => audio_stream
                .stream
                .play()
                .map_err(|e| failure::err_msg(format!("Feedback::run error : {}", e))),
            None => Err(failure::err_msg(
                "Feedback::run error : no open audio stream",
            )),
        }
    }

    fn close(&mut self) -> Result<(), failure::Error> {
        self.audio_stream = None;
        Ok(())
    }

    fn backup(&self) -> (&str, &str, String) {
        (output::KIND, MODEL, String::new())
    }
}
