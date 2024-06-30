use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use haversine_calculator::{calc::naive_haversine, generate::CoordSerializer, CoordPair};
use rand::{rngs::StdRng, SeedableRng};

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
    Calculate {},
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
        Commands::Calculate {} => {
            let reader = BufReader::new(File::open(path)?);
            let res: Vec<CoordPair> = serde_json::from_reader(reader).unwrap();
            println!("Finished parsing json");
            let mut running_sum = 0.0;
            let len = res.len();
            for cp in res {
                let res = naive_haversine(cp);
                running_sum += res;
            }
            println!("The avg is: {}", running_sum / len as f64);
        }
    }
    Ok(())
}
