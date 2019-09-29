extern crate cpal;
extern crate failure;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::sync::mpsc::{Receiver, Sender};

use crate::audio_data::{AudioOutput, Interleaved};

pub struct Playback {
    //    event_loop: cpal::EventLoop,
    //    format: cpal::Format,
    //  stream_id: cpal::StreamId,
    sender: std::sync::mpsc::Sender<Interleaved>,
    //receiver: std::sync::mpsc::Receiver<Interleaved>,
    //    data: Interleaved,
}

impl Playback {
    pub fn new() -> Result<Self, failure::Error> {
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
                                            nc = ad.channels();
                                            len = ad.samples() * nc;
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
            sender: sender,
            //      receiver: receiver,
            //            data: Interleaved::new(0, 0),
        })
    }
}

impl AudioOutput for Playback {
    fn open(&self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn write(&self, data: Interleaved) -> Result<(), failure::Error> {
        self.sender.send(data).unwrap();
        Ok(())
    }

    fn close(&self) -> Result<(), failure::Error> {
        self.sender.send(Interleaved::end()).unwrap();
        Ok(())
    }
}
