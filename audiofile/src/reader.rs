use ffmpeg::{codec, filter, format, frame, media, util};

fn filter(
    decoder: &codec::decoder::Audio,
    sample_format: format::Sample, sample_rate: usize
) -> Result<filter::Graph, ffmpeg::Error> {
    let mut filter = filter::Graph::new();

    let args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    filter.add(&filter::find("abuffer").unwrap(), "in", &args)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

    {
        let mut out = filter.get("out").unwrap();

        out.set_sample_format(sample_format);
        out.set_channel_layout(decoder.channel_layout());
        out.set_sample_rate(sample_rate as u32);
    }

    filter.output("in", 0)?.input("out", 0)?.parse("anull")?;
    filter.validate()?;

    println!("{}", filter.dump());

    Ok(filter)
}

pub struct Reader {
    stream_index: usize,
    decoder: codec::decoder::Audio,
    input_context: format::context::Input,
    in_time_base: ffmpeg::Rational,
    filter: filter::Graph,
    frame: frame::audio::Audio,
    remaining_samples_count: usize,
    remaining_samples_index: usize,
}

impl Reader {
    pub fn new(file_path: &str, sample_rate: usize) -> Result<Reader, failure::Error> {
        let input_context = format::input(&file_path).unwrap();
        let input_stream = input_context
        .streams()
        .best(media::Type::Audio).ok_or(failure::err_msg("could not find best audio stream"))?;
    
        let context = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
        let mut decoder = context.decoder().audio()?;

        decoder.set_parameters(input_stream.parameters())?;

        let sample_format = format::Sample::F32(util::format::sample::Type::Planar);
        let filter = filter(&decoder, sample_format, sample_rate).map_err(|e| failure::err_msg(format!("Filter graph : {}", e)))?;

        let mut frame = frame::audio::Audio::new(sample_format, 1024, decoder.channel_layout());
        frame.set_rate(sample_rate as u32);

        let in_time_base = decoder.time_base();

        Ok(Self {
            stream_index: input_stream.index(),
            decoder,
            input_context,
            in_time_base,
            filter,
            frame,
            remaining_samples_count: 0,
            remaining_samples_index: 0,
        })
    }
    
    pub fn channels(&self) -> usize {
        self.decoder.channels() as usize
    }

    pub fn read_samples(&mut self, channels: &mut Vec<Vec<f32>>, nb_samples_per_channel: usize,) -> Result<usize, failure::Error> {
        let nb_channels = channels.len().min(self.channels());
        let mut sample_idx = 0;
        let mut decoded_frame = frame::Audio::empty();

        while sample_idx < nb_samples_per_channel {
            let rem_len = self.remaining_samples_count.min(nb_samples_per_channel - sample_idx);

            if rem_len > 0 {
                for chan_idx in 0..nb_channels {
                    let channel = &mut channels[chan_idx];
                    let plane = self.frame.plane(chan_idx);

                    for i in 0..rem_len {
                        channel[sample_idx + i] = plane[self.remaining_samples_index + i];
                    }
                }
                self.remaining_samples_count -= rem_len;
                self.remaining_samples_index += rem_len;
                sample_idx += rem_len;
            }
            else if self.filter.get("out").unwrap().sink().frame(&mut self.frame).is_ok() {
                self.remaining_samples_count = self.frame.samples();
                self.remaining_samples_index = 0;
            }
            else if self.decoder.receive_frame(&mut decoded_frame).is_ok() {
                let timestamp = decoded_frame.timestamp();
                decoded_frame.set_pts(timestamp);
                self.filter.get("in").unwrap().source().add(&decoded_frame).map_err(|e| failure::err_msg(format!("{}", e)))?;
            }
            else if let Some((stream, mut packet)) = self.input_context.packets().next() {
                if stream.index() == self.stream_index {
                    packet.rescale_ts(stream.time_base(), self.in_time_base);
                    self.decoder.send_packet(&packet)?;
                }
            }
            else {
                return Ok(sample_idx);
            }
        }
 
        Ok(sample_idx)
    }
}
