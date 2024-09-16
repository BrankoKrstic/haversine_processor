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

pub fn deserialize_single_pass(mut s: &str) -> Result<Vec<CoordPair>, DeserializationError> {
    if &s[..2] != "[{" || &s[s.len() - 2..] != "}]" {
        return Err(DeserializationError(
            "Unexpected opening character".to_string(),
        ));
    }
    s = &s[2..s.len() - 2];
    let mut out = Vec::with_capacity(s.len() / 100);
    for item in s.split("},{") {
        let mut lat0: Option<f64> = None;
        let mut lon0: Option<f64> = None;
        let mut lat1: Option<f64> = None;
        let mut lon1: Option<f64> = None;
        for item in item.split(',') {
            let (key, val) = item
                .split_once(':')
                .ok_or(DeserializationError("Invalid json".to_string()))?;
            let val = val.parse::<f64>().map_err(|_| {
                DeserializationError("Can't parse floating point value".to_string())
            })?;
            match key {
                "\"lat0\"" => lat0 = Some(val),
                "\"lon0\"" => lon0 = Some(val),
                "\"lat1\"" => lat1 = Some(val),
                "\"lon1\"" => lon1 = Some(val),
                _ => {}
            }
        }

        let cp = CoordPair {
            lat0: lat0.ok_or_else(|| DeserializationError("member lat0 missing".to_owned()))?,
            lon0: lon0.ok_or_else(|| DeserializationError("member lon0 missing".to_owned()))?,
            lat1: lat1.ok_or_else(|| DeserializationError("member lat1 missing".to_owned()))?,
            lon1: lon1.ok_or_else(|| DeserializationError("member lon1 missing".to_owned()))?,
        };
        out.push(cp);
    }
    Ok(out)
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
        record_bytes(1);
        drop(handle);
        if next_byte[0] != b'[' {
            return Err(DeserializationError(format!(
                "Unexpected opening character '{}'",
                next_byte[0] as char
            )));
        }
        let mut out = Vec::new();

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
        unsafe {
            println!("{} {}", MIN_SIZE, MAX_SIZE);
        }

        Ok(out)
    }
}

static mut MIN_SIZE: usize = usize::MAX;
static mut MAX_SIZE: usize = 0;

impl Deserializable for CoordPair {
    fn streaming_deserialize(reader: &mut impl BufRead) -> Result<Self, DeserializationError> {
        let mut lat0: Option<f64> = None;
        let mut lon0: Option<f64> = None;
        let mut lat1: Option<f64> = None;
        let mut lon1: Option<f64> = None;
        let mut buf = vec![];
        bench_block!(handle, "Deserialize Read");
        let read = reader.read_until(b'}', &mut buf)?;
        unsafe {
            if read < MIN_SIZE {
                MIN_SIZE = read;
            }
            if read > MAX_SIZE {
                MAX_SIZE = read;
            }
        }
        record_bytes(read as u64);
        drop(handle);
        if buf[0] != b'{' {
            return Err(DeserializationError(format!(
                "Unexpected opening character '{}'",
                buf[0] as char
            )));
        }

        let mut buf_slice = &buf[1..buf.len() - 1];
        while !buf_slice.is_empty() {
            bench_block!(handle, "Process Key Val Pair");
            let next_colon = buf_slice.iter().position(|&b| b == b':').unwrap();
            let key_slice = &buf_slice[0..next_colon];
            buf_slice = &buf_slice[next_colon + 1..];
            let next_comma = buf_slice
                .iter()
                .position(|&b| b == b',')
                .unwrap_or(buf_slice.len());
            let val_slice = &buf_slice[..next_comma];
            drop(handle);
            bench_block!(handle, "Process UTF8");
            let key = unsafe { std::str::from_utf8_unchecked(key_slice).trim() };
            let val_as_utf8 = unsafe { std::str::from_utf8_unchecked(val_slice).trim() };
            drop(handle);

            bench_block!(handle, "Parse float");
            let val = val_as_utf8.parse::<f64>().map_err(|_| {
                DeserializationError("Can't parse floating point value".to_string())
            })?;
            drop(handle);

            if next_comma == buf_slice.len() {
                buf_slice = &buf_slice[next_comma..];
            } else {
                buf_slice = &buf_slice[next_comma + 1..];
            }
            bench_block!(handle, "Match Key");
            match key {
                "\"lat0\"" => lat0 = Some(val),
                "\"lon0\"" => lon0 = Some(val),
                "\"lat1\"" => lat1 = Some(val),
                "\"lon1\"" => lon1 = Some(val),
                _ => {}
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
        let mut buf = ryu::Buffer::new();
        writer.write_all(b"{\"lat0\":")?;
        writer.write_all(buf.format(self.lat0).as_bytes())?;
        writer.write_all(b",\"lon0\":")?;
        writer.write_all(buf.format(self.lon0).as_bytes())?;
        writer.write_all(b",\"lon1\":")?;
        writer.write_all(buf.format(self.lon1).as_bytes())?;
        writer.write_all(b",\"lat1\":")?;
        writer.write_all(buf.format(self.lat1).as_bytes())?;

        writer.write_all(b"}")
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
