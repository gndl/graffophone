use std::collections::HashMap;
use std::str::FromStr;

use talker::identifier::{Id, Index};

use crate::mixer;
use crate::output;

pub struct PTalkerVoice {
    pub talker: Id,
    pub voice_port: usize,
}
pub enum PTalk {
    TalkerVoice(PTalkerVoice),
    Value(f32),
}

pub struct PConnection {
    pub ear_idx: usize,
    pub set_idx: usize,
    pub hum_idx: usize,
    pub talk_idx: usize,
    pub talk: PTalk,
}
pub struct PTalker<'a> {
    pub model: &'a str,
    pub id: Id,
    pub name: &'a str,
    pub data: Option<&'a str>,
    pub connections: Vec<PConnection>,
}
pub struct PMixer<'a> {
    pub talker: PTalker<'a>,
    pub outputs: Vec<Id>,
}
pub struct POutput<'a> {
    pub model: &'a str,
    pub id: Id,
    pub name: &'a str,
    pub data: Option<&'a str>,
}

fn id_from_str(id_str: &str) -> Result<Id, failure::Error> {
    match Id::from_str(id_str) {
        Ok(id) => Ok(id),
        Err(e) => Err(failure::err_msg(format!(
            "Failed to get id from {} : {}!",
            id_str,
            e.to_string()
        ))),
    }
}

fn parse_id_name<'a>(source: &'a str) -> Result<(&'a str, Id, &'a str), failure::Error> {
    let desc_end = source.find("\n").unwrap_or(source.len());

    let (id_desc, name) = if let Some(id_desc_end) = source.find("#") {
        (
            source.get(..id_desc_end).unwrap(),
            source.get(id_desc_end + "#".len()..desc_end).unwrap(),
        )
    } else {
        (source.get(..desc_end).unwrap(), "")
    };
    Ok((
        source.get(desc_end + "\n".len()..).unwrap(),
        id_from_str(id_desc)?,
        name,
    ))
}

fn parse_data<'a>(source: &'a str) -> Result<(&'a str, Option<&'a str>), failure::Error> {
    if source.starts_with("[:") {
        let data_end = source.find(":]\n").unwrap();
        let data = source.get("[:".len()..data_end).unwrap();
        Ok((source.get(data_end + ":]\n".len()..).unwrap(), Some(data)))
    } else {
        Ok((source, None))
    }
}

fn parse_connections<'a>(source: &'a str) -> Result<(&'a str, Vec<PConnection>), failure::Error> {
    let mut connections = Vec::new();
    let mut src = source;

    while src.starts_with(">") {
        src = src.get(">".len()..).unwrap();

        let ear_desc_end = src.find("<").unwrap();
        let mut ear_desc = src.get(..ear_desc_end).unwrap();

        let (ear_idx, set_idx, hum_idx, talk_idx) = if let Some(ear_id_end) = ear_desc.find(".") {
            let ear_id = ear_desc.get(..ear_id_end).unwrap();
            let ear_idx = Index::from_str(ear_id).unwrap_or(0);
            ear_desc = ear_desc.get(ear_id_end + ".".len()..).unwrap();

            if let Some(ear_set_id_end) = ear_desc.find(".") {
                let set_id = ear_desc.get(..ear_set_id_end).unwrap();
                let set_idx = Index::from_str(set_id).unwrap_or(0);
                ear_desc = ear_desc.get(ear_set_id_end + ".".len()..).unwrap();

                if let Some(ear_hum_id_end) = ear_desc.find(".") {
                    let hum_id = ear_desc.get(..ear_hum_id_end).unwrap();
                    let hum_idx = Index::from_str(hum_id).unwrap_or(0);

                    let talk_id = ear_desc.get(ear_hum_id_end + ".".len()..).unwrap();
                    let talk_idx = Index::from_str(talk_id).unwrap_or(0);
                    (ear_idx, set_idx, hum_idx, talk_idx)
                } else {
                    (ear_idx, set_idx, Index::from_str(ear_desc).unwrap_or(0), 0)
                }
            } else {
                (ear_idx, Index::from_str(ear_desc).unwrap_or(0), 0, 0)
            }
        } else {
            (Index::from_str(ear_desc).unwrap_or(0), 0, 0, 0)
        };

        let talk_desc_end = src.find("\n").unwrap();
        let talk_desc = src.get(ear_desc_end + "<".len()..talk_desc_end).unwrap();

        let talk = match f32::from_str(talk_desc) {
            Ok(value) => PTalk::Value(value),
            Err(_) => {
                if let Some(talker_desc_end) = talk_desc.find(":") {
                    let talker_desc = talk_desc.get(..talker_desc_end).unwrap();
                    let voice_id = talk_desc.get(talker_desc_end + ":".len()..).unwrap();
                    let voice_port = Index::from_str(voice_id).unwrap_or(0);
                    PTalk::TalkerVoice(PTalkerVoice {
                        talker: id_from_str(talker_desc)?,
                        voice_port,
                    })
                } else {
                    PTalk::TalkerVoice(PTalkerVoice {
                        talker: id_from_str(talk_desc)?,
                        voice_port: 0,
                    })
                }
            }
        };

        let cnx = PConnection {
            ear_idx,
            set_idx,
            hum_idx,
            talk_idx,
            talk,
        };
        connections.push(cnx);
        src = src.get(talk_desc_end + "\n".len()..).unwrap();
    }
    Ok((src, connections))
}

fn parse_outputs<'a>(source: &'a str) -> Result<(&'a str, Vec<Id>), failure::Error> {
    let mut outputs = Vec::new();
    let mut src = source;

    while src.starts_with("< ") {
        src = src.get("< ".len()..).unwrap();

        let output_id_desc_end = src.find("\n").unwrap();
        let output_id_desc = src.get(..output_id_desc_end).unwrap();
        let id = id_from_str(output_id_desc)?;
        outputs.push(id);
        src = src.get(output_id_desc_end + "\n".len()..).unwrap();
    }
    Ok((src, outputs))
}

pub fn parse<'a>(
    source: &'a String,
) -> Result<
    (
        HashMap<Id, PTalker<'a>>,
        HashMap<Id, PMixer<'a>>,
        HashMap<Id, POutput<'a>>,
    ),
    failure::Error,
> {
    let mut talkers = HashMap::new();
    let mut mixers = HashMap::new();
    let mut outputs = HashMap::new();

    let mut src = source.as_str();
    let mixer_tag = format!("{} ", mixer::KIND);
    let output_tag = format!("{} ", output::KIND);

    while src.len() > 0 {
        if src.starts_with("\n") {
            src = src.get("\n".len()..).unwrap();
        } else if src.starts_with(&mixer_tag) {
            let (rest, id, name) = parse_id_name(src.get(mixer_tag.len()..).unwrap())?;
            let (rest, connections) = parse_connections(rest)?;
            let (rest, outputs) = parse_outputs(rest)?;

            let mixer = PMixer {
                talker: PTalker {
                    model: mixer::KIND,
                    id,
                    name,
                    data: None,
                    connections,
                },
                outputs,
            };
            mixers.insert(id, mixer);
            src = rest;
        } else if src.starts_with(&output_tag) {
            src = src.get(output_tag.len()..).unwrap();
            let model_end = src.find(" ").unwrap();
            let model = src.get(..model_end).unwrap();

            let (rest, id, name) = parse_id_name(src.get(model_end + " ".len()..).unwrap())?;
            let (rest, data) = parse_data(rest)?;

            let output = POutput {
                model,
                id,
                name,
                data,
            };
            outputs.insert(id, output);
            src = rest;
        } else if let Some(model_end) = src.find(" ") {
            let model = src.get(..model_end).unwrap();
            let (rest, id, name) = parse_id_name(src.get(model_end + " ".len()..).unwrap())?;
            let (rest, data) = parse_data(rest)?;
            let (rest, connections) = parse_connections(rest)?;

            let talker = PTalker {
                model,
                id,
                name,
                data,
                connections,
            };
            talkers.insert(id, talker);
            src = rest;
        } else {
            src = src.get("\n".len()..).unwrap();
        }
    }

    Ok((talkers, mixers, outputs))
}
