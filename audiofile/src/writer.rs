use ffmpeg::{channel_layout, codec, filter, format, frame, util};


fn filter(sample_format: &format::Sample, in_sample_rate: usize,
    channel_layout: &channel_layout::ChannelLayout,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg::Error> {
    let mut filter = filter::Graph::new();

    let in_spec = format!("time_base=1/{}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        in_sample_rate, in_sample_rate, sample_format.name(), channel_layout.bits());

    filter.add(&filter::find("abuffer").unwrap(), "in", &in_spec)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;
    {
        let mut out = filter.get("out").unwrap();
        
        out.set_sample_format(encoder.format());
        out.set_channel_layout(encoder.channel_layout());
        out.set_sample_rate(encoder.rate());
    }

    let conversion_spec = format!("aresample={},aformat=sample_fmts={}:channel_layouts=0x{:x}",
            encoder.rate(), encoder.format().name(), encoder.channel_layout().bits());

    filter.output("in", 0)?.input("out", 0)?.parse(&conversion_spec)?;
    filter.validate()?;

    if let Some(codec) = encoder.codec() {
        if !codec
        .capabilities()
        .contains(ffmpeg::codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
        {
            filter
            .get("out")
            .unwrap()
            .sink()
            .set_frame_size(encoder.frame_size());
        }
    }

    Ok(filter)
}

pub struct Writer {
    frame: frame::audio::Audio,
    filter: filter::Graph,
    encoder: codec::encoder::Audio,
    output: format::context::Output,
    in_time_base: ffmpeg::Rational,
    out_time_base: ffmpeg::Rational,
    pts: i64,
}

impl Writer {
    pub fn new(codec_name: &str, in_sample_rate: usize, out_sample_rate: usize, channels: usize, file_path: &str,) -> Result<Writer, failure::Error> {

        let mut output = format::output(&file_path)?;

        let codec = ffmpeg::encoder::find_by_name(codec_name)
            .ok_or(failure::err_msg(format!("Failed to find encoder {}", codec_name)))?
            .audio()?;

        let global = output
            .format()
            .flags()
            .contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

        let mut stream = output.add_stream(codec)?;

        let codec_context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let mut encoder = codec_context.encoder().audio()?;

        let codec_channel_layout = codec
            .channel_layouts()
            .map(|cls| cls.best(channels as i32))
            .unwrap_or(ffmpeg::channel_layout::ChannelLayout::STEREO);

        if global {
            encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
        }

        encoder.set_rate(out_sample_rate as i32);
        encoder.set_channel_layout(codec_channel_layout);
        #[cfg(not(feature = "ffmpeg_7_0"))]
        {
            encoder.set_channels(codec_channel_layout.channels());
        }

        for fmt in codec.formats().expect("unknown supported formats") {
            println!("Codec {} ({}) supported format : {}", codec.name(), codec.description(), fmt.name());
        }

        encoder.set_format(codec.formats().expect("unknown supported formats").last().unwrap());
        encoder.set_bit_rate(96000);
        encoder.set_max_bit_rate(192000);

        encoder.set_time_base((1, out_sample_rate as i32));
        stream.set_time_base((1, out_sample_rate as i32));

        let encoder = encoder.open_as(codec)?;
        stream.set_parameters(&encoder);

        let sample_format = format::Sample::F32(util::format::sample::Type::Planar);
        let filter = filter(&sample_format, in_sample_rate, &codec_channel_layout, &encoder).map_err(|e| failure::err_msg(format!("Filter graph : {}", e)))?;
        let mut frame = frame::audio::Audio::new(sample_format, 1024, codec_channel_layout);
        frame.set_rate(in_sample_rate as u32);

        let in_time_base = ffmpeg::Rational::new(1, in_sample_rate as i32);
        let out_time_base = stream.time_base();

        Ok(Self {frame, filter, encoder, output, in_time_base, out_time_base, pts: 0})
    }
    
    pub fn channels(&self) -> usize {
        self.encoder.channels() as usize
    }

    pub fn write_header(&mut self) -> Result<(), failure::Error> {
        self.output.write_header().map_err(|e| failure::err_msg(format!("{}", e)))
    }

    pub fn write_samples(&mut self, channels: &Vec<Vec<f32>>, nb_samples_per_channel: usize,) -> Result<(), failure::Error> {

        if self.frame.samples() != nb_samples_per_channel {
            let mut frame = frame::audio::Audio::new(
                format::Sample::F32(util::format::sample::Type::Planar), 
                nb_samples_per_channel,
                self.encoder.channel_layout());

                frame.set_rate(self.frame.rate());
                
                self.frame = frame;
        }

        let last_in_chan = channels.len() - 1;
        let mut in_chan_idx = 0;

        for plane_idx in 0..self.frame.planes() {
            let channel = &channels[in_chan_idx];
            let plane = self.frame.plane_mut(plane_idx);

            for i in 0..nb_samples_per_channel {
                plane[i] = channel[i];
            }

            if in_chan_idx < last_in_chan {
                in_chan_idx += 1;
            }
        }

        self.frame.set_rate(self.in_time_base.denominator() as u32);
        self.frame.set_pts(Some(self.pts));
        self.pts = self.pts + nb_samples_per_channel as i64;

        self.filter.get("in").unwrap().source().add(&self.frame).map_err(|e| failure::err_msg(format!("{}", e)))?;
        self.get_and_process_filtered_frames()
    }

    fn flush_filter(&mut self) -> Result<(), failure::Error> {
        self.filter.get("in").unwrap().source().flush().map_err(|e| failure::err_msg(format!("{}", e)))
    }

    fn get_and_process_filtered_frames(&mut self) -> Result<(), failure::Error> {
        let mut filtered_frame = frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered_frame)
            .is_ok()
        {
            self.encoder.send_frame(&filtered_frame).map_err(|e| failure::err_msg(format!("{}", e)))?;
            self.receive_and_process_encoded_packets()?;
        }
        Ok(())
    }

    fn send_eof_to_encoder(&mut self) -> Result<(), failure::Error> {
        self.encoder.send_eof().map_err(|e| failure::err_msg(format!("{}", e)))
    }

    fn receive_and_process_encoded_packets(&mut self) -> Result<(), failure::Error> {
        let mut packet = ffmpeg::Packet::empty();

        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.rescale_ts(self.in_time_base, self.out_time_base);
            packet.write_interleaved(&mut self.output).map_err(|e| failure::err_msg(format!("{}", e)))?;
        }
        Ok(())
    }

    pub fn write_trailer(&mut self) -> Result<(), failure::Error> {
        self.output.write_trailer().map_err(|e| failure::err_msg(format!("{}", e)))
    }

    pub fn close(&mut self) -> Result<(), failure::Error> {
        let _ = self.flush_filter();
        let _ = self.get_and_process_filtered_frames();
        let _ = self.send_eof_to_encoder();
        let _ = self.receive_and_process_encoded_packets();
        self.write_trailer()
    }
}
