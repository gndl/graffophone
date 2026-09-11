use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::path::Path;
use std::path::PathBuf;

use audiofile::writer::Writer;
use talker::identifier::{Id, RIdentifier};

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
    folder_path: PathBuf,
    file_name: PathBuf,
    writer: Option<Writer>,
}

impl AudioFileOutput {
    pub fn new(
        id: Id,
        codec_name: &str,
        in_sample_rate: usize,
        out_sample_rate: usize,
        channel_layout: &str,
        folder_path: &Path,
        file_name: &Path,
    ) -> Result<AudioFileOutput, failure::Error> {
        Ok(Self {
            identifier: output::new_identifier(id, "", MODEL),
            codec_name: codec_name.to_string(),
            in_sample_rate,
            out_sample_rate,
            channel_layout: channel_layout.to_string(),
            folder_path: folder_path.to_path_buf(),
            file_name: file_name.to_path_buf(),
            writer: None,
        })
    }

    pub fn new_ref(
        id: Id,
        codec_name: &str,
        in_sample_rate: usize,
        out_sample_rate: usize,
        channel_layout: &str,
        folder_path: &Path,
        file_name: &Path,
    ) -> Result<ROutput, failure::Error> {
        Ok(Rc::new(RefCell::new(AudioFileOutput::new(
            id,
            codec_name,
            in_sample_rate,
            out_sample_rate,
            channel_layout,
            folder_path,
            file_name
        )?)))
    }

    pub fn from_backup(id: Id, in_sample_rate: usize, configuration: &str, folder_path: &Path,) -> Result<ROutput, failure::Error> {
        let params: Vec<&str> = configuration.split('|').collect();

        if params.len() == 4 {
            let codec_name = params[0];
            let out_sample_rate = usize::from_str(params[1]).map_err(|e| failure::err_msg(format!("{}", e)))?;
            let channel_layout = params[2];
            let file_name = PathBuf::from(params[3]);
            AudioFileOutput::new_ref(id, codec_name, in_sample_rate, out_sample_rate, channel_layout, folder_path, file_name.as_path())
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

    fn channels_count(&self) -> usize {
        match &self.writer {
            Some(ctx) => ctx.channels(),
            None => channel::Layout::channels_count(&self.channel_layout),
        }
    }

    fn channels_names(&self) -> Vec<&'static str> {
        channel::Layout::channels_names(&self.channel_layout)
    }

    fn file_path(&self) -> PathBuf {
        self.folder_path.join(&self.file_name)
    }

    fn open(&mut self) -> Result<(), failure::Error> {

        let channels_count = channel::Layout::channels_count(&self.channel_layout);
        let file_path = self.file_path();

        let mut writer = Writer::new(
            self.codec_name.as_str(),
            self.in_sample_rate,
            self.out_sample_rate,
            channels_count,
            &file_path,
        )?;

        if writer.channels() != channels_count {
            self.channel_layout = channel::Layout::from_channels_count(writer.channels()).to_string();
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
        let conf = format!(
            "{}|{}|{}|{}",
            self.codec_name(),
            self.out_sample_rate,
            self.channel_layout,
            self.file_name.to_string_lossy()
        );
        (output::KIND, MODEL, conf)
    }
}
