use std::cell::RefCell;
use std::rc::Rc;

use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf;
use ringbuf::RingBuffer;

use talker::identifier::RIdentifier;

use crate::audio_data::Vector;
use crate::output;
use crate::output::{Output, ROutput};

pub const MODEL: &str = "feedback";

pub type AudioProducer = ringbuf::Producer<f32>;

pub struct AudioStream {
    stream: cpal::Stream,
    producer: AudioProducer,
}
pub struct Feedback {
    identifier: RIdentifier,
    nb_channels: usize,
    nb_samples: usize,
    audio_stream: Option<AudioStream>,
}

impl Feedback {
    pub fn new(_nb_channels: usize, nb_samples: usize) -> Result<Feedback, failure::Error> {
        // Default devices.
        let output_device = cpal::default_host()
            .default_output_device()
            .expect("failed to get default output device");

        println!("Using default output device: \"{}\"", output_device.name()?);

        let config: cpal::StreamConfig = output_device.default_output_config()?.into();

        let nb_channels = config.channels as usize;

        Ok(Self {
            identifier: output::new_identifier("", MODEL),
            nb_channels,
            nb_samples,
            audio_stream: None,
        })
    }

    pub fn new_ref(nb_channels: usize, nb_samples: usize) -> Result<ROutput, failure::Error> {
        Ok(Rc::new(RefCell::new(Feedback::new(
            nb_channels,
            nb_samples,
        )?)))
    }

    fn make_audio_stream(
        nb_channels: usize,
        nb_samples: usize,
    ) -> Result<AudioStream, failure::Error> {
        let output_device = cpal::default_host()
            .default_output_device()
            .expect("failed to get default output device");

        let config: cpal::StreamConfig = output_device.default_output_config()?.into();

        let latency_samples = nb_samples * nb_channels;

        // The buffer to share samples
        let ring = RingBuffer::new(latency_samples * 5);
        let (producer, mut consumer) = ring.split();

        let output_data_fn = move |data: &mut [f32]| {
            for sample in data {
                let mut ov = consumer.pop();

                while ov == None {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    ov = consumer.pop();
                }
                *sample = ov.unwrap_or(0.0);
            }
        };

        let err_fn = |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        let stream = output_device.build_output_stream(&config, output_data_fn, err_fn)?;

        stream
            .play()
            .map_err(|e| failure::err_msg(format!("Feedback::open error : {}", e)))?;

        Ok(AudioStream { producer, stream })
    }
}

impl Output for Feedback {
    fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }

    fn model(&self) -> &str {
        MODEL
    }

    fn nb_channels(&self) -> usize {
        self.nb_channels
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

                for i in 0..nb_samples_per_channel {
                    for ch in channels {
                        let sample = ch[i];
                        while output_fell_behind < 30 && audio_stream.producer.push(sample).is_err()
                        {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            output_fell_behind = output_fell_behind + 1;
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

    fn backup(&self) -> (&str, &str, Vec<(&str, String)>) {
        (output::KIND, MODEL, Vec::new())
    }
}
