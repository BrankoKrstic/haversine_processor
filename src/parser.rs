use std::{
    error::Error,
    fmt::Display,
    io::{BufRead, Write},
};

use rand::RngCore;

use crate::{bench_block, generate::CoordPairGen, metrics::record_bytes, CoordPair};

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

pub fn serialize<S: Serializable>(
    obj: &mut S,
    writer: &mut impl Write,
) -> Result<(), std::io::Error> {
    bench_block!("Serialize Data to Json");
    obj.streaming_serialize(writer)
}

impl<T> Deserializable for Vec<T>
where
    T: Deserializable,
{
    fn streaming_deserialize(reader: &mut impl BufRead) -> Result<Self, DeserializationError> {
        let mut next_byte = [0u8; 1];
        bench_block!(handle, "Deserialize Read");
        reader.read_exact(&mut next_byte[..])?;
        drop(handle);
        record_bytes(1);
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
                bench_block!(handle, "Deserialize Read");
                reader.read_exact(&mut next_byte[..])?;
                record_bytes(1);
                drop(handle);

                match next_byte[0] {
                    b',' => break,
                    b']' => break 'outer,
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
        bench_block!(handle, "Deserialize Read");
        let read = reader.read_until(b'}', &mut buf)?;
        record_bytes(read as u64);
        drop(handle);
        if buf[0] != b'{' {
            return Err(DeserializationError(format!(
                "Unexpected opening character '{}'",
                buf[0] as char
            )));
        }
        bench_block!(handle, "Process UTF8");

        let value = std::str::from_utf8(&buf[1..buf.len() - 1])
            .map_err(|_| DeserializationError("Only utf8 format supported".to_string()))?;
        drop(handle);

        for item in value.split(',') {
            bench_block!(handle, "Process Key Val Pair");

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
            drop(handle);
            bench_block!(handle, "Match Key");

            match key.trim() {
                "\"lat0\"" => lat0 = Some(val),
                "\"lon0\"" => lon0 = Some(val),
                "\"lat1\"" => lat1 = Some(val),
                "\"lon1\"" => lon1 = Some(val),
                _ => {
                    println!("{}", key);
                }
            }
            drop(handle);
        }
        let out = CoordPair {
            lat0: lat0.ok_or_else(|| DeserializationError("member lat0 missing".to_owned()))?,
            lon0: lon0.ok_or_else(|| DeserializationError("member lon0 missing".to_owned()))?,
            lat1: lat1.ok_or_else(|| DeserializationError("member lat1 missing".to_owned()))?,
            lon1: lon1.ok_or_else(|| DeserializationError("member lon1 missing".to_owned()))?,
        };
        Ok(out)
    }
}

impl Serializable for CoordPair {
    fn streaming_serialize(&mut self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all(
            format!(
                "{{\"lat0\":{},\"lon0\":{},\"lat1\":{},\"lon1\":{}}}",
                self.lat0, self.lon0, self.lat1, self.lon1
            )
            .as_bytes(),
        )
    }
}

pub trait Serializable {
    fn streaming_serialize(&mut self, writer: &mut impl Write) -> Result<(), std::io::Error>;
}

impl<T: RngCore> Serializable for CoordPairGen<T> {
    fn streaming_serialize(&mut self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all(b"[")?;

        if let Some(mut item) = self.next() {
            item.streaming_serialize(writer)?;
        }
        for mut item in self {
            writer.write_all(b",")?;
            item.streaming_serialize(writer)?;
        }
        writer.write_all(b"]")?;

        Ok(())
    }
}
