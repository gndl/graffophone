use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

extern crate failure;

extern crate cpal;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

use gpplugin::horn::AudioBuf;
use gpplugin::identifier::RIdentifier;

use crate::audio_data::{Interleaved, Vector};
use crate::output;
use crate::output::{Output, ROutput};

pub const MODEL: &str = "playback";

pub struct Playback {
    identifier: RIdentifier,
    sender: Sender<Interleaved>,
    join_handle: JoinHandle<()>,
    nb_channels: usize,
    nb_samples: usize,
}

impl Playback {
    pub fn new(nb_channels: usize, nb_samples: usize) -> Result<Playback, failure::Error> {
        let (sender, receiver): (Sender<Interleaved>, Receiver<Interleaved>) =
            std::sync::mpsc::channel();

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err(failure::err_msg("failed to find a default output device")),
        };

        let format = device.default_output_format()?;
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_output_stream(&device, &format)?;

        event_loop.play_stream(stream_id.clone())?;

        let join_handle: JoinHandle<()> = std::thread::spawn(move || {
            let nb_channels = format.channels as usize;
            let mut nc = 0;
            let mut len = 0;
            let mut av = Vec::new();
            let mut pos = 0;

            event_loop.run(move |id, result| {
                let out_data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("Error on stream {:?} match date result : {}", id, err);
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
                                            return;
                                        } else {
                                            nc = ad.nb_channels();
                                            len = ad.nb_samples_per_channel() * nc;
                                            av = ad.vector();
                                            pos = 0;
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "Error on stream {:?} receiver.recv() : {}",
                                            id, err
                                        );
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
            identifier: output::new_identifier("", MODEL),
            sender,
            join_handle,
            nb_channels,
            nb_samples,
        })
    }

    pub fn new_ref(nb_channels: usize, nb_samples: usize) -> Result<ROutput, failure::Error> {
        Ok(Rc::new(RefCell::new(Playback::new(
            nb_channels,
            nb_samples,
        )?)))
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

impl Output for Playback {
    fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }

    fn open(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn write(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        let ad = Interleaved::new(channels, nb_samples_per_channel);
        self.sender
            .send(ad)
            .map_err(|e| failure::err_msg(format!("Playback::write error : {}", e)))
    }

    fn close(&mut self) -> Result<(), failure::Error> {
        match self.sender.send(Interleaved::end()) {
            Ok(()) => Ok(()),
            /*self
            .join_handle
            .join()
            .map_err(|_| failure::err_msg(format!("Playback::close error on send end"))),*/
            Err(e) => Err(failure::err_msg(format!(
                "Playback::close error : {}",
                e.description()
            ))),
        }
    }

    fn backup(&self) -> (&str, &str, Vec<(&str, String)>) {
        (output::KIND, MODEL, Vec::new())
    }
}
