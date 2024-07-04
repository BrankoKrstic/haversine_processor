use std::{arch::x86_64::_rdtsc, collections::HashMap};

#[cfg(feature = "bench")]
static mut BENCHMARK_ANCHOR: BenchmarkAnchor = BenchmarkAnchor::new();

#[cfg(feature = "bench")]
struct BenchmarkAnchor {
    data: [(u64, u64, &'static str); 1024],
    depth: usize,
    byte_count: usize,
    used: Vec<usize>,
}

#[cfg(feature = "bench")]
impl BenchmarkAnchor {
    const fn new() -> Self {
        Self {
            data: [(0, 0, ""); 1024],
            depth: 0,
            byte_count: 0,
            used: vec![],
        }
    }
}

#[cfg(feature = "bench")]
pub struct Benchmark {
    start_os: std::time::Instant,
    start_cpu: u64,
}
#[cfg(feature = "bench")]
impl Benchmark {
    pub fn init() -> Self {
        Self {
            start_os: std::time::Instant::now(),
            start_cpu: read_cpu_timer(),
        }
    }
}

#[cfg(feature = "bench")]
impl Drop for Benchmark {
    fn drop(&mut self) {
        let start_cpu = self.start_cpu;
        let end_cpu = read_cpu_timer();
        let total_time = self.start_os.elapsed().as_nanos();
        let cpu_freq = (end_cpu - start_cpu) as u128 * os_freq() as u128 / total_time;
        let byte_count = unsafe { BENCHMARK_ANCHOR.byte_count };
        println!(
            "Total time: {}ms (CPU freq {})",
            total_time as f64 / 1000000.0,
            cpu_freq
        );
        if byte_count != 0 {
            let time_spent_reading_in_secs = (end_cpu - start_cpu) as f64 / cpu_freq as f64;
            let bytes_per_second = byte_count as f64 / time_spent_reading_in_secs;
            println!("Total processed: {:.2}Mb", byte_count as f64 / 1_000_000.0);
            println!("Read speed: {:.2}Gbps", bytes_per_second / 1_000_000_000.0);
        }
        let mut bench_map = HashMap::new();

        let mut total_cpu_used = 0;
        unsafe {
            for i in BENCHMARK_ANCHOR.used.iter() {
                let item = BENCHMARK_ANCHOR.data[*i];
                let entry = bench_map.entry(item.2).or_insert((0, 0));
                entry.0 += item.0;
                entry.1 += item.1;
                total_cpu_used += item.0;
            }
        }

        for (name, vals) in bench_map {
            let (cur_cpu, child_cpu) = vals;
            let cpu_percentage = cur_cpu as f64 / total_cpu_used as f64 * 100.0;
            let child_cpu_percentage = child_cpu as f64 / total_cpu_used as f64 * 100.0;
            if child_cpu_percentage != 0.0 {
                println!(
                    "{name}: {cur_cpu} cycles ({:.2}%), {child_cpu} ({:.2}%) from children",
                    cpu_percentage, child_cpu_percentage
                );
            } else {
                println!("{name}: {cur_cpu} ({:.2}%)", cpu_percentage);
            }
        }
    }
}

#[cfg(not(feature = "bench"))]
pub struct Benchmark;

#[cfg(not(feature = "bench"))]
impl Benchmark {
    pub fn init() -> Self {
        Self
    }
}

pub fn os_freq() -> u64 {
    1_000_000_000
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

#[cfg(feature = "bench")]
pub struct BenchmarkOnDrop {
    start: u64,
    slot: usize,
}
#[cfg(feature = "bench")]
impl BenchmarkOnDrop {
    pub fn new(name: &'static str, slot: usize) -> Self {
        unsafe {
            BENCHMARK_ANCHOR.depth += 1;
            BENCHMARK_ANCHOR.data[slot].2 = name;
            BENCHMARK_ANCHOR.used.push(slot)
        }

        Self {
            start: read_cpu_timer(),
            slot,
        }
    }
}
#[cfg(feature = "bench")]
impl Drop for BenchmarkOnDrop {
    fn drop(&mut self) {
        unsafe {
            let end_time = read_cpu_timer() - self.start;
            BENCHMARK_ANCHOR.depth -= 1;
            BENCHMARK_ANCHOR.data[self.slot].0 += end_time * (BENCHMARK_ANCHOR.depth == 0) as u64;
            BENCHMARK_ANCHOR.data[self.slot].1 += end_time * (BENCHMARK_ANCHOR.depth == 1) as u64;
        }
    }
}

#[cfg(not(feature = "bench"))]
pub struct BenchmarkOnDrop;

#[cfg(not(feature = "bench"))]
impl BenchmarkOnDrop {
    pub fn new(_: &'static str, _: usize) -> Self {
        Self
    }
}
#[cfg(not(feature = "bench"))]
impl Drop for BenchmarkOnDrop {
    fn drop(&mut self) {}
}

pub fn record_bytes_read(bytes: usize) {
    unsafe { BENCHMARK_ANCHOR.byte_count += bytes };
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
