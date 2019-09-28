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

fn example() -> Result<(), failure::Error> {
    descibe_devices()?;
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let format = device.default_output_format()?;
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_output_stream(&device, &format)?;
    event_loop.play_stream(stream_id.clone())?;

    let sample_rate = format.sample_rate.0 as f32;
    let mut sample_clock = 0f32;

    // Produce a sinusoid of maximum amplitude.
    let mut next_value = || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
    };

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
                buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            _ => (),
        }
    });
}

fn descibe_devices() -> Result<(), failure::Error> {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        //        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;
        let default_in = host.default_input_device().map(|e| e.name().unwrap());
        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        println!("  Default Input Device:\n    {:?}", default_in);
        println!("  Default Output Device:\n    {:?}", default_out);

        let devices = host.devices()?;
        println!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            println!("  {}. \"{}\"", device_index + 1, device.name()?);

            // Input formats
            if let Ok(fmt) = device.default_input_format() {
                println!("    Default input stream format:\n      {:?}", fmt);
            }
            let mut input_formats = match device.supported_input_formats() {
                Ok(f) => f.peekable(),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if input_formats.peek().is_some() {
                println!("    All supported input stream formats:");
                for (format_index, format) in input_formats.enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        format_index + 1,
                        format
                    );
                }
            }

            // Output formats
            if let Ok(fmt) = device.default_output_format() {
                println!("    Default output stream format:\n      {:?}", fmt);
            }
            let mut output_formats = match device.supported_output_formats() {
                Ok(f) => f.peekable(),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if output_formats.peek().is_some() {
                println!("    All supported output stream formats:");
                for (format_index, format) in output_formats.enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        format_index + 1,
                        format
                    );
                }
            }
        }
    }

    Ok(())
}
