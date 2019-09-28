extern crate cpal;
extern crate failure;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::sync::mpsc::{Receiver, Sender};

use crate::audio_data::{AudioOutput, Interleaved};

pub struct Playback {
    //    event_loop: cpal::EventLoop,
    //    format: cpal::Format,
    //    stream_id: cpal::StreamId,
    sender: std::sync::mpsc::Sender<Interleaved>,
    //    receiver: std::sync::mpsc::Receiver<Interleaved>,
    //    data: Interleaved,
}

impl Playback {
    pub fn new() -> Result<Self, failure::Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let format = device.default_output_format()?;
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_output_stream(&device, &format)?;
        let (sender, receiver): (Sender<Interleaved>, Receiver<Interleaved>) =
            std::sync::mpsc::channel();

        event_loop.play_stream(stream_id.clone())?;

        std::thread::spawn(move || {
            let nb_channels = format.channels as usize;
            let ad = receiver.recv().unwrap();
            let mut nc = ad.channels();
            let mut len = ad.samples() * nc;
            let mut av = ad.vector();
            let mut pos = 0;
            //            let len = data.len;
            /*

            let next_value = |chan: usize| {
                if pos >= av.len() {
                    let ad = receiver.recv().unwrap();
            nc = ad.0;
             av = ad.1;
                    pos = 0;
                }
                let chan = chan % nc;

                let v = data.sample(chan, pos);
                pos += 1;
                v
                0.0
                    av[nc * pos + chan]
            };
                 */

            event_loop.run(move |id, result| {
                let data = match result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", id, err);
                        return;
                    }
                };

                match data {
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                    } => {
                        for sample in buffer.chunks_mut(nb_channels) {
                            if pos >= len {
                                let ad = receiver.recv().unwrap();
                                nc = ad.channels();
                                len = ad.samples() * nc;
                                av = ad.vector();
                                pos = 0;
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
            //          event_loop: event_loop,
            //            format: format,
            //           stream_id: stream_id,
            sender: sender,
            //            receiver: receiver,
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
        //        self.event_loop.destroy_stream(self.stream_id);
        Ok(())
    }
}
