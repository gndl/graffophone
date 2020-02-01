extern crate cpal;
extern crate failure;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use gpplugin::horn::AudioBuf;
use std::sync::mpsc::{Receiver, Sender};

use crate::audio_data::{AudioOutput, Interleaved, Vector};

pub struct Playback {
    //    event_loop: cpal::EventLoop,
    //    format: cpal::Format,
    //  stream_id: cpal::StreamId,
    sender: std::sync::mpsc::Sender<Interleaved>,
    //receiver: std::sync::mpsc::Receiver<Interleaved>,
    //    data: Interleaved,
    nb_channels: usize,
    nb_samples: usize,
}

impl Playback {
    pub fn new(nb_channels: usize, nb_samples: usize) -> Result<Self, failure::Error> {
        let (sender, receiver): (Sender<Interleaved>, Receiver<Interleaved>) =
            std::sync::mpsc::channel();

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err(failure::err_msg("failed to find a default output device")),
        };
        //            .expect("failed to find a default output device");
        let format = device.default_output_format()?;
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_output_stream(&device, &format)?;

        event_loop.play_stream(stream_id.clone())?;

        std::thread::spawn(move || {
            let nb_channels = format.channels as usize;
            let mut nc = 0;
            let mut len = 0;
            let mut av = Vec::new();
            let mut pos = 0;

            event_loop.run(move |id, result| {
                let out_data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                        return;
                    }
                };

                match out_data {
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                    } => {
                        for sample in buffer.chunks_mut(nb_channels) {
                            if pos >= len {
                                match receiver.recv() {
                                    Ok(ad) => {
                                        if ad.is_end() {
                                            //                                            event_loop.destroy_stream(stream_id);
                                            return;
                                        } else {
                                            nc = ad.nb_channels();
                                            len = ad.nb_samples_per_channel() * nc;
                                            av = ad.vector();
                                            pos = 0;
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                                        return;
                                    }
                                }
                            }
                            for chan in 0..nb_channels {
                                sample[chan] = av[pos + chan];
                            }
                            pos += nc;
                        }
                    }
                    _ => (),
                }
            });
        });

        Ok(Self {
            //            event_loop: event_loop,
            //            format: format,
            //            stream_id: stream_id,
            sender,
            //      receiver: receiver,
            //            data: Interleaved::new(0, 0),
            nb_channels,
            nb_samples,
        })
    }

    pub fn write_mono(&mut self, audio_buf: &AudioBuf, len: usize) -> Result<(), failure::Error> {
        let audio_buffer_slice = audio_buf.get();

        let mut right: Vec<f32> = vec![0.; len];
        let mut left: Vec<f32> = vec![0.; len];

        for i in 0..len {
            let sample = audio_buffer_slice[i].get();
            right[i] = sample;
            left[i] = sample;
        }

        let mut channels = Vec::with_capacity(2);
        channels.push(right);
        channels.push(left);
        self.write(&channels, len)
    }
}

impl AudioOutput for Playback {
    fn open(&self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn write(
        &self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        let ad = Interleaved::new(channels, nb_samples_per_channel);
        self.sender.send(ad).unwrap();
        Ok(())
    }

    fn close(&self) -> Result<(), failure::Error> {
        self.sender.send(Interleaved::end()).unwrap();
        Ok(())
    }
}
