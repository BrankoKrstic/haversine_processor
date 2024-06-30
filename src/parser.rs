use std::{error::Error, fmt::Display, io::BufRead};

use crate::CoordPair;

/// Pretty much serde without the intermediate representation

#[derive(Debug)]
pub struct DeserializationError(String);

impl From<std::io::Error> for DeserializationError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for DeserializationError {}

pub trait Deserializable
where
    Self: Sized,
{
    fn streaming_deserialize(reader: &mut impl BufRead) -> Result<Self, DeserializationError>;
}

pub fn deserialize<D: Deserializable>(
    reader: &mut impl BufRead,
) -> Result<D, DeserializationError> {
    D::streaming_deserialize(reader)
}

impl<T> Deserializable for Vec<T>
where
    T: Deserializable,
{
    fn streaming_deserialize(reader: &mut impl BufRead) -> Result<Self, DeserializationError> {
        let mut next_byte = [0u8; 1];
        reader.read_exact(&mut next_byte[..])?;
        if next_byte[0] != b'[' {
            return Err(DeserializationError(format!(
                "Unexpected opening character '{}'",
                next_byte[0] as char
            )));
        }
        let mut out = vec![];

        'outer: loop {
            out.push(T::streaming_deserialize(reader)?);
            loop {
                reader.read_exact(&mut next_byte[..])?;
                match next_byte[0] {
                    b']' => break 'outer,
                    b',' => break,
                    x if x.is_ascii_whitespace() => {}
                    x => {
                        return Err(DeserializationError(format!(
                            "Unexpected byte '{}'",
                            x as char
                        )))
                    }
                }
            }
        }

        Ok(out)
    }
}

impl Deserializable for CoordPair {
    fn streaming_deserialize(reader: &mut impl BufRead) -> Result<Self, DeserializationError> {
        let mut lat0: Option<f64> = None;
        let mut lon0: Option<f64> = None;
        let mut lat1: Option<f64> = None;
        let mut lon1: Option<f64> = None;
        let mut buf = vec![];
        reader.read_until(b'}', &mut buf)?;
        let value = String::from_utf8(buf)
            .map_err(|_| DeserializationError("Only utf8 format supported".to_string()))?;
        for item in value.split(',') {
            let mut key_val = item.split(':');
            let key = key_val
                .next()
                .ok_or(DeserializationError("Unexpected format".to_owned()))?;
            let val = key_val
                .next()
                .ok_or(DeserializationError("Unexpected format".to_owned()))?;

            let val = val.trim().parse::<f64>().map_err(|_| {
                DeserializationError(format!("Can't parse floating point value from {}", val))
            })?;
            match key.trim() {
                "\"lat0\"" => lat0 = Some(val),
                "\"lon0\"" => lon0 = Some(val),
                "\"lat1\"" => lat1 = Some(val),
                "\"lon1\"" => lon1 = Some(val),
                _ => {}
            }
        }
        let out = CoordPair {
            lat0: lat0.unwrap_or(Err(DeserializationError("member lat0 missing".to_owned()))?),
            lon0: lon0.unwrap_or(Err(DeserializationError("member lon0 missing".to_owned()))?),
            lat1: lat1.unwrap_or(Err(DeserializationError("member lat1 missing".to_owned()))?),
            lon1: lon1.unwrap_or(Err(DeserializationError("member lon1 missing".to_owned()))?),
        };
        Ok(out)
    }
}
