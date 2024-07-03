use std::arch::x86_64::_rdtsc;

#[cfg(feature = "bench")]
static mut BENCHMARK_ANCHOR: BenchmarkAnchor = BenchmarkAnchor::new();

#[cfg(feature = "bench")]
struct BenchmarkAnchor {
    data: [(u64, u64, &'static str); 1024],
    next: usize,
    depth: usize,
}

#[cfg(feature = "bench")]
impl BenchmarkAnchor {
    const fn new() -> Self {
        Self {
            data: [(0, 0, ""); 1024],
            next: 0,
            depth: 0,
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

        println!(
            "Total time: {}ms (CPU freq {})",
            total_time as f64 / 1000000.0,
            cpu_freq
        );
        let op_count = unsafe { BENCHMARK_ANCHOR.next };
        let total_cpu_used: u64 = unsafe {
            BENCHMARK_ANCHOR
                .data
                .iter()
                .take(BENCHMARK_ANCHOR.next)
                .map(|(cycles, _, _)| cycles)
                .sum()
        };
        for i in 0..op_count {
            let (cur_cpu, child_cpu, name) = unsafe { BENCHMARK_ANCHOR.data[i] };
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
    name: &'static str,
}
#[cfg(feature = "bench")]
impl BenchmarkOnDrop {
    pub fn new(name: &'static str) -> Self {
        unsafe {
            BENCHMARK_ANCHOR.depth += 1;
        }
        Self {
            start: read_cpu_timer(),
            name,
        }
    }
}
#[cfg(feature = "bench")]
impl Drop for BenchmarkOnDrop {
    fn drop(&mut self) {
        unsafe {
            let end_time = read_cpu_timer() - self.start;
            BENCHMARK_ANCHOR.depth -= 1;
            BENCHMARK_ANCHOR.data[BENCHMARK_ANCHOR.next].0 = end_time;
            BENCHMARK_ANCHOR.data[BENCHMARK_ANCHOR.next].2 = self.name;
            BENCHMARK_ANCHOR.data[BENCHMARK_ANCHOR.next].1 +=
                end_time * (BENCHMARK_ANCHOR.depth == 1) as u64;

            BENCHMARK_ANCHOR.next += (BENCHMARK_ANCHOR.depth == 0) as usize;
        }
    }
}

#[cfg(not(feature = "bench"))]
pub struct BenchmarkOnDrop;

#[cfg(not(feature = "bench"))]
impl BenchmarkOnDrop {
    pub fn new(_: &'static str) -> Self {
        Self
    }
}
#[cfg(not(feature = "bench"))]
impl Drop for BenchmarkOnDrop {
    fn drop(&mut self) {}
}

#[macro_export]
macro_rules! bench_block {
    ($handle:ident, $name:literal) => {
        let $handle = $crate::metrics::BenchmarkOnDrop::new($name);
    };
    ($name:literal) => {
        let _bench_container = $crate::metrics::BenchmarkOnDrop::new($name);
    };
}
