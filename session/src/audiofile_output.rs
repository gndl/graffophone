use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use audiofile::writer::Writer;
use talker::identifier::RIdentifier;

use crate::audio_data::Vector;
use crate::channel;
use crate::output;
use crate::output::{Output, ROutput};

pub const MODEL: &str = "file";



pub struct AudioFileOutput {
    identifier: RIdentifier,
    codec_name: String,
    in_sample_rate: usize,
    out_sample_rate: usize,
    channel_layout: String,
    file_path: String,
    writer: Option<Writer>,
}

impl AudioFileOutput {
    pub fn new(codec_name: &str, in_sample_rate: usize, out_sample_rate: usize, channel_layout: &str, file_path: &str,) -> Result<AudioFileOutput, failure::Error> {
        Ok(Self {
            identifier: output::new_identifier("", MODEL),
            codec_name: codec_name.to_string(),
            in_sample_rate,
            out_sample_rate,
            channel_layout: channel_layout.to_string(),
            file_path: file_path.to_string(),
            writer: None,
        })
    }

    pub fn new_ref(codec_name: &str, in_sample_rate: usize, out_sample_rate: usize, channel_layout: &str, file_path: &str,) -> Result<ROutput, failure::Error> {
        Ok(Rc::new(RefCell::new(AudioFileOutput::new(codec_name, in_sample_rate, out_sample_rate, channel_layout, file_path)?)))
    }

    pub fn from_backup(in_sample_rate: usize, configuration: &str,) -> Result<ROutput, failure::Error> {
        let params: Vec<&str> = configuration.split('|').collect();

        if params.len() == 4 {
            let codec_name = params[0];
            let out_sample_rate = usize::from_str(params[1]).map_err(|e| failure::err_msg(format!("{}", e)))?;
            let channel_layout = params[2];
            let file_path = params[3];
            AudioFileOutput::new_ref(codec_name, in_sample_rate, out_sample_rate, channel_layout, file_path)
        }
        else {
            Err(failure::err_msg(format!("AudioFileOutput configuration {} need 4 parameters!", configuration)))
        }
    }
}

impl Output for AudioFileOutput {
    fn identifier<'a>(&'a self) -> &'a RIdentifier {
        &self.identifier
    }

    fn model(&self) -> String{
        MODEL.to_string()
    }

    fn codec_name<'a>(&'a self) -> &'a str {
        &self.codec_name
    }

    fn sample_rate(&self) -> usize {
        self.out_sample_rate
    }

    fn channel_layout<'a>(&'a self) -> &'a str{
        &self.channel_layout
    }

    fn channels(&self) -> usize {
        match &self.writer {
            Some(ctx) => ctx.channels(),
            None => channel::Layout::channels(&self.channel_layout),
        }
    }

    fn channels_names(&self) -> Vec<&'static str> {
        channel::Layout::channels_names(&self.channel_layout)
    }

    fn file_path<'a>(&'a self) -> &'a str {
        &self.file_path
    }

    fn open(&mut self) -> Result<(), failure::Error> {

        let channels = channel::Layout::channels(&self.channel_layout);

        let mut writer = Writer::new(self.codec_name.as_str(), self.in_sample_rate, self.out_sample_rate, channels, self.file_path.as_str())?;

        if writer.channels() != channels {
            self.channel_layout = channel::Layout::from_channels(writer.channels()).to_string();
        }

        writer.write_header()?;

        self.writer = Some(writer);

        Ok(())
    }

    fn write(
        &mut self,
        channels: &Vec<Vector>,
        nb_samples_per_channel: usize,
    ) -> Result<(), failure::Error> {
        let writer = self.writer.as_mut().ok_or(failure::err_msg(format!("AudioFileOutput not open")))?;

        writer.write_samples(channels, nb_samples_per_channel)
    }

    fn pause(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn run(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), failure::Error> {
        let ctx = self.writer.as_mut().ok_or(failure::err_msg(format!("AudioFileOutput not open")))?;

        let res = ctx.close();
        self.writer = None;
        res
    }

    fn backup(&self) -> (&str, &str, String) {
        let conf = format!("{}|{}|{}|{}", self.codec_name(), self.out_sample_rate, self.channel_layout, self.file_path());
        (output::KIND, MODEL, conf)
    }
}
