extern crate failure;
use std::str::FromStr;

pub enum Data {
    Nil,
    Int(i64),
    Float(f32),
    String(String),
    Text(String),
    File(String),
}

impl Data {
    pub fn type_str(&self) -> &'static str {
        match self {
            Data::Int(_) => "Int",
            Data::Float(_) => "Float",
            Data::String(_) => "String",
            Data::Text(_) => "Text",
            Data::File(_) => "File",
            Data::Nil => "Nil",
        }
    }
    pub fn notify_incompatibility(&self, expected_type: &str) -> failure::Error {
        eprintln!(
            "Type {} is incompatible with the expected {} type",
            self.type_str(),
            expected_type
        );
        failure::err_msg("Value type is incompatible with the expected type")
    }

    pub fn to_i(&self) -> Result<i64, failure::Error> {
        match self {
            Data::Int(i) => Ok(*i),
            _ => Err(self.notify_incompatibility("Int")),
        }
    }
    pub fn to_f(&self) -> Result<f32, failure::Error> {
        match self {
            Data::Float(f) => Ok(*f),
            _ => Err(self.notify_incompatibility("Float")),
        }
    }
    pub fn to_s(&self) -> Result<String, failure::Error> {
        match self {
            Data::String(s) => Ok(s.to_string()),
            _ => Err(self.notify_incompatibility("String")),
        }
    }
    pub fn to_t(&self) -> Result<String, failure::Error> {
        match self {
            Data::Text(t) => Ok(t.to_string()),
            _ => Err(self.notify_incompatibility("Text")),
        }
    }
    pub fn to_fl(&self) -> Result<String, failure::Error> {
        match self {
            Data::File(f) => Ok(f.to_string()),
            _ => Err(self.notify_incompatibility("File")),
        }
    }

    pub fn i(i: i64) -> Self {
        Data::Int(i)
    }
    pub fn f(f: f32) -> Self {
        Data::Float(f)
    }
    pub fn s(s: String) -> Self {
        Data::String(s)
    }
    pub fn t(t: String) -> Self {
        Data::Text(t)
    }
    pub fn fl(f: String) -> Self {
        Data::File(f)
    }

    pub fn to_string(&self) -> String {
        match self {
            Data::Int(i) => i.to_string(),
            Data::Float(f) => f.to_string(),
            Data::String(s) => s.to_string(),
            Data::Text(s) => s.to_string(),
            Data::File(s) => s.to_string(),
            Data::Nil => "".to_string(),
        }
    }

    fn notify_string_incompatibility(&self, s: &str) -> Result<Data, failure::Error> {
        eprintln!(
            "Value string {} is incompatible with the expected {} type",
            s,
            self.type_str()
        );
        Err(failure::err_msg(
            "Value string is incompatible with the expected type",
        ))
    }

    pub fn birth(&self, s: &str) -> Result<Self, failure::Error> {
        match self {
            Data::Int(_) => match i64::from_str(s) {
                Ok(i) => Ok(Data::Int(i)),
                _ => self.notify_string_incompatibility(s),
            },
            Data::Float(_) => match f32::from_str(s) {
                Ok(f) => Ok(Data::Float(f)),
                _ => self.notify_string_incompatibility(s),
            },
            Data::String(_) => Ok(Data::String(s.to_string())),
            Data::Text(_) => Ok(Data::Text(s.to_string())),
            Data::File(_) => Ok(Data::File(s.to_string())),
            Data::Nil => Ok(Data::Nil),
        }
    }
}
