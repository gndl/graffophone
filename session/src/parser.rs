use std::collections::HashMap;
use std::str::FromStr;

use talker::identifier::{Id, Index};

pub struct PTalkerVoice<'a> {
    pub talker: Id,
    pub voice: &'a str,
}
pub enum PTalk<'a> {
    TalkerVoice(PTalkerVoice<'a>),
    Value(f32),
}

pub struct PConnection<'a> {
    pub ear_tag: &'a str,
    pub set_idx: usize,
    pub hum_tag: &'a str,
    pub talk_idx: usize,
    pub talk: PTalk<'a>,
}
pub struct PTalker<'a> {
    pub model: &'a str,
    pub id: Id,
    pub name: &'a str,
    pub data: Option<&'a str>,
    pub connections: Vec<PConnection<'a>>,
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

fn parse_connections<'a>(
    source: &'a str,
) -> Result<(&'a str, Vec<PConnection<'a>>), failure::Error> {
    let mut connections = Vec::new();
    let mut src = source;

    while src.starts_with("> ") {
        src = src.get("> ".len()..).unwrap();

        let ear_desc_end = src.find(" <- ").unwrap();
        let mut ear_desc = src.get(..ear_desc_end).unwrap();

        let (ear_tag, set_idx, hum_tag, talk_idx) = if let Some(ear_tag_end) = ear_desc.find(".") {
            let ear_tag = ear_desc.get(..ear_tag_end).unwrap();
            ear_desc = ear_desc.get(ear_tag_end + ".".len()..).unwrap();

            if let Some(ear_set_end) = ear_desc.find(".") {
                let set_desc = ear_desc.get(..ear_set_end).unwrap();
                let set_idx = Index::from_str(set_desc).unwrap_or(0);
                ear_desc = ear_desc.get(ear_set_end + ".".len()..).unwrap();

                if let Some(ear_hum_end) = ear_desc.find(".") {
                    let hum_tag = ear_desc.get(..ear_hum_end).unwrap();
                    let talk_idx =
                        Index::from_str(ear_desc.get(ear_hum_end + ".".len()..).unwrap())
                            .unwrap_or(0);
                    (ear_tag, set_idx, hum_tag, talk_idx)
                } else {
                    (ear_tag, set_idx, ear_desc, 0)
                }
            } else {
                (ear_tag, Index::from_str(ear_desc).unwrap_or(0), "", 0)
            }
        } else {
            (ear_desc, 0, "", 0)
        };

        let talk_desc_end = src.find("\n").unwrap();
        let talk_desc = src.get(ear_desc_end + " <- ".len()..talk_desc_end).unwrap();

        let talk = match f32::from_str(talk_desc) {
            Ok(value) => PTalk::Value(value),
            Err(_) => {
                if let Some(talker_desc_end) = talk_desc.find(":") {
                    let talker_desc = talk_desc.get(..talker_desc_end).unwrap();
                    let voice = talk_desc.get(talker_desc_end + ":".len()..).unwrap();
                    PTalk::TalkerVoice(PTalkerVoice {
                        talker: id_from_str(talker_desc)?,
                        voice,
                    })
                } else {
                    PTalk::TalkerVoice(PTalkerVoice {
                        talker: id_from_str(talk_desc)?,
                        voice: &"",
                    })
                }
            }
        };

        let cnx = PConnection {
            ear_tag,
            set_idx,
            hum_tag,
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

    while src.len() > 0 {
        if src.starts_with("\n") {
            src = src.get("\n".len()..).unwrap();
        } else if src.starts_with("mixer ") {
            let (rest, id, name) = parse_id_name(src.get("mixer ".len()..).unwrap())?;
            let (rest, connections) = parse_connections(rest)?;
            let (rest, outputs) = parse_outputs(rest)?;

            let mixer = PMixer {
                talker: PTalker {
                    model: "mixer",
                    id,
                    name,
                    data: None,
                    connections,
                },
                outputs,
            };
            mixers.insert(id, mixer);
            src = rest;
        } else if src.starts_with("output ") {
            src = src.get("output ".len()..).unwrap();
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
