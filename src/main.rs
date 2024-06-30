use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use haversine_calculator::{
    bench_block,
    calc::naive_haversine,
    generate::CoordPairGen,
    metrics::Bench,
    parser::{deserialize, serialize},
    CoordPair,
};
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
    let mut benchmark = Bench::<5>::start();
    let cli = Cli::parse();
    let path = PathBuf::from(cli.filename);
    match cli.command {
        Commands::Generate {
            count,
            seed,
            uniform,
        } => {
            bench_block!("Generate Haversine Input");
            let rng = StdRng::seed_from_u64(seed);
            let mut coord_pair_generator = CoordPairGen::new(rng, !uniform, count);
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            serialize(&mut coord_pair_generator, &mut writer)?;
        }
        Commands::Calculate {} => {
            let mut reader = BufReader::new(File::open(path)?);
            benchmark.bench("Startup");
            let res: Vec<CoordPair> = deserialize(&mut reader).unwrap();
            benchmark.bench("Deserialization");
            let mut running_sum = 0.0;
            let len = res.len();
            benchmark.bench("Misc Setup");
            for cp in res {
                let res = naive_haversine(cp);
                running_sum += res;
            }
            let result = running_sum / len as f64;
            benchmark.bench("Haversine Calculation");
            println!("The avg is: {}", result);
            benchmark.bench("Output");
            benchmark.end();
        }
    }
    Ok(())
}
