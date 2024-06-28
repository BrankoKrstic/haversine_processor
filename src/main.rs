use std::{
    cell::RefCell,
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{ser::SerializeSeq, Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE", default_value_t = String::from("./input.json"))]
    filename: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(short, long, default_value_t = 10_000)]
        count: usize,
        #[arg(short, long, default_value_t = 1212121212)]
        seed: u64,
        #[arg(short, long, default_value_t = false)]
        uniform: bool,
    },
    Run {},
}

fn gen_rand_lat_lng(
    min_lat: f64,
    max_lat: f64,
    min_lng: f64,
    max_lng: f64,
    rng: &mut impl Rng,
) -> (f64, f64) {
    let lat = rng.gen_range(min_lat..max_lat);
    let lng = rng.gen_range(min_lng..max_lng);
    (lat, lng)
}

#[derive(Debug, Serialize, Deserialize)]
struct CoordPair {
    lat0: f64,
    lng0: f64,
    lat1: f64,
    lng1: f64,
}

impl From<((f64, f64), (f64, f64))> for CoordPair {
    fn from(value: ((f64, f64), (f64, f64))) -> Self {
        Self {
            lat0: value.0 .0,
            lng0: value.0 .1,
            lat1: value.1 .0,
            lng1: value.1 .1,
        }
    }
}

struct CoordPairGen<T> {
    cur_item: usize,
    item_count: usize,
    min_lat: f64,
    max_lat: f64,
    min_lng: f64,
    max_lng: f64,
    should_cluster: bool,
    rng: T,
}

impl<T: Rng> CoordPairGen<T> {
    fn new(rng: T, should_cluster: bool, item_count: usize) -> Self {
        Self {
            cur_item: 0,
            item_count,
            rng,
            min_lat: -90.0,
            max_lat: 90.0,
            min_lng: -180.0,
            max_lng: 180.0,
            should_cluster,
        }
    }
}

impl<T: Rng> Iterator for CoordPairGen<T> {
    type Item = CoordPair;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item_count < self.cur_item {
            return None;
        }
        if self.should_cluster && self.cur_item % 1000 == 0 {
            todo!();
        }
        self.cur_item += 1;
        Some(
            (
                gen_rand_lat_lng(
                    self.min_lat,
                    self.max_lat,
                    self.min_lng,
                    self.max_lng,
                    &mut self.rng,
                ),
                gen_rand_lat_lng(
                    self.min_lat,
                    self.max_lat,
                    self.min_lng,
                    self.max_lng,
                    &mut self.rng,
                ),
            )
                .into(),
        )
    }
}

struct CoordSerializer<T> {
    inner: RefCell<CoordPairGen<T>>,
}

impl<T: Rng> CoordSerializer<T> {
    fn new(rng: T, should_cluster: bool, item_count: usize) -> Self {
        Self {
            inner: RefCell::new(CoordPairGen::new(rng, should_cluster, item_count)),
        }
    }
}

impl<T: Rng> Serialize for CoordSerializer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut iter = self.inner.borrow_mut();
        let mut seq = serializer.serialize_seq(Some(iter.item_count - iter.cur_item))?;
        for item in iter.by_ref() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();
    let path = PathBuf::from(cli.filename);
    match cli.command {
        Commands::Generate {
            count,
            seed,
            uniform,
        } => {
            let rng = StdRng::seed_from_u64(seed);
            let serializer = CoordSerializer::new(rng, !uniform, count);
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &serializer).unwrap();
        }
        Commands::Run {} => todo!(),
    }
    Ok(())
}
