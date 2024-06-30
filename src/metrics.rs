use std::{arch::x86_64::_rdtsc, mem::MaybeUninit, time::Instant};

pub fn os_freq() -> u64 {
    1_000_000_000
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub struct Bench<'a, const N: usize> {
    start_os: Instant,
    start_cpu: u64,
    checkpoints: [MaybeUninit<(&'a str, u64)>; N],
    cur_checkpoint: usize,
}

impl<'a, const N: usize> Bench<'a, N> {
    pub fn start() -> Self {
        Self {
            start_os: Instant::now(),
            start_cpu: read_cpu_timer(),
            checkpoints: [MaybeUninit::uninit(); N],
            cur_checkpoint: 0,
        }
    }
    pub fn bench(&mut self, name: &'a str) {
        debug_assert!(self.cur_checkpoint < N);
        self.checkpoints[self.cur_checkpoint].write((name, read_cpu_timer()));
        self.cur_checkpoint += 1;
    }
    pub fn end(&mut self) {
        debug_assert!(self.cur_checkpoint == N);
        let start_cpu = self.start_cpu;
        let end_cpu = unsafe { self.checkpoints[N - 1].assume_init().1 };
        let total_time = self.start_os.elapsed().as_nanos();
        let total_cpu = end_cpu - start_cpu;
        let cpu_freq = total_cpu as u128 * os_freq() as u128 / total_time;
        if N == 1 {
            let (name, _) = unsafe { self.checkpoints[0].assume_init() };
            println!(
                "Bench {name} time: {}ms cycles: {total_cpu} (CPU freq {cpu_freq})",
                total_time as f64 / 1000000.0
            );
            return;
        }
        println!(
            "Total time: {}ms (CPU freq {})",
            total_time as f64 / 1000000.0,
            cpu_freq
        );
        let mut prev = start_cpu;
        for i in 0..N {
            let (name, cur_cpu) = unsafe { self.checkpoints[i].assume_init() };
            let segment_cpu = cur_cpu - prev;
            let cpu_percentage = segment_cpu as f64 / total_cpu as f64 * 100.0;
            println!("{name}: {segment_cpu} ({:.2})%", cpu_percentage);
            prev = cur_cpu;
        }
    }
}

pub struct BenchmarkOnDrop<'a> {
    bench: Bench<'a, 1>,
    name: &'a str,
}

impl<'a> BenchmarkOnDrop<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            bench: Bench::start(),
            name,
        }
    }
}

impl<'a> Drop for BenchmarkOnDrop<'a> {
    fn drop(&mut self) {
        self.bench.bench(self.name);
        self.bench.end();
    }
}

#[macro_export]
macro_rules! bench_block {
    ($name:literal) => {
        let _bench_container = $crate::metrics::BenchmarkOnDrop::new($name);
    };
}
