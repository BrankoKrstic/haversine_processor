use std::{arch::x86_64::_rdtsc, collections::HashMap};

static mut BENCHMARK_ANCHOR: BenchmarkAnchor = BenchmarkAnchor::new();

static mut BYTE_COUNT: u64 = 0;

static mut CHILD_CPU_SPEND: u64 = 0;

struct BenchmarkAnchor {
    data: [(u64, u64, u64, &'static str); 1024],
    depth: usize,
}

impl BenchmarkAnchor {
    const fn new() -> Self {
        Self {
            data: [(0, 0, 0, ""); 1024],
            depth: 0,
        }
    }
}

pub struct Benchmark {
    start_os: std::time::Instant,
    start_cpu: u64,
}
impl Benchmark {
    pub fn init() -> Self {
        Self {
            start_os: std::time::Instant::now(),
            start_cpu: read_cpu_timer(),
        }
    }
}

impl Drop for Benchmark {
    fn drop(&mut self) {
        let start_cpu = self.start_cpu;
        let end_cpu = read_cpu_timer();
        let total_time = self.start_os.elapsed().as_nanos();
        let cpu_freq = (end_cpu - start_cpu) as u128 * os_freq() as u128 / total_time;
        println!(
            "Total time: {}ms (CPU freq {})",
            total_time as f64 / 1000000.0,
            cpu_freq
        );
        let mut bench_map = HashMap::new();

        let mut total_cpu_used = 0;
        unsafe {
            for item in BENCHMARK_ANCHOR.data.iter().filter(|item| item.0 != 0) {
                let entry = bench_map.entry(item.3).or_insert((0, 0, 0));
                entry.0 += item.0;
                entry.1 += item.1;
                entry.2 += item.2;
                total_cpu_used += item.0;
            }
        }

        for (name, vals) in bench_map {
            let (cur_cpu, child_cpu, bytes_processed) = vals;
            let cpu_percentage = cur_cpu as f64 / total_cpu_used as f64 * 100.0;
            let child_cpu_percentage = child_cpu as f64 / total_cpu_used as f64 * 100.0;
            if child_cpu_percentage != 0.0 {
                print!(
                    "{name}: {cur_cpu} cycles ({:.2}%), {child_cpu} ({:.2}%) from children",
                    cpu_percentage, child_cpu_percentage
                );
            } else {
                print!("{name}: {cur_cpu} ({:.2}%)", cpu_percentage);
            }
            if bytes_processed != 0 {
                let time_spent_reading_in_secs = cur_cpu as f64 / cpu_freq as f64;
                let bytes_per_second = bytes_processed as f64 / time_spent_reading_in_secs;
                print!(
                    " Total processed: {:.2}Mb (at {:.2}Gbps)",
                    bytes_processed as f64 / 1_000_000.0,
                    bytes_per_second / 1_000_000_000.0
                );
            }
            println!();
        }
    }
}

pub fn os_freq() -> u64 {
    1_000_000_000
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub struct BenchmarkOnDrop {
    start: u64,
    slot: usize,
}
impl BenchmarkOnDrop {
    pub fn new(name: &'static str, slot: usize) -> Self {
        unsafe {
            BENCHMARK_ANCHOR.depth += 1;
            BENCHMARK_ANCHOR.data[slot].3 = name;
        }

        Self {
            start: read_cpu_timer(),
            slot,
        }
    }
}
impl Drop for BenchmarkOnDrop {
    fn drop(&mut self) {
        unsafe {
            BENCHMARK_ANCHOR.depth -= 1;
            let end_time = read_cpu_timer() - self.start;
            let child_cpu = CHILD_CPU_SPEND * (BENCHMARK_ANCHOR.depth == 0) as u64;
            let byte_count = BYTE_COUNT * (BENCHMARK_ANCHOR.depth == 0) as u64;

            BENCHMARK_ANCHOR.data[self.slot].0 += end_time * (BENCHMARK_ANCHOR.depth == 0) as u64;
            BENCHMARK_ANCHOR.data[self.slot].1 += child_cpu;
            BENCHMARK_ANCHOR.data[self.slot].2 += byte_count;

            CHILD_CPU_SPEND -= child_cpu;
            CHILD_CPU_SPEND += end_time * (BENCHMARK_ANCHOR.depth == 1) as u64;
            BYTE_COUNT -= byte_count;
        }
    }
}

pub fn record_bytes(bytes: u64) {
    unsafe { BYTE_COUNT += bytes };
}

#[macro_export]
macro_rules! bench_block {
    ($handle:ident, $name:literal) => {
        let $handle = $crate::metrics::BenchmarkOnDrop::new($name, ::counter::counter!());
    };
    ($name:literal) => {
        let _bench_container = $crate::metrics::BenchmarkOnDrop::new($name, ::counter::counter!());
    };
}
