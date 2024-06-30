use lazy_static::lazy_static;
use std::{arch::x86_64::_rdtsc, mem::MaybeUninit, time::Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub fn os_freq() -> u64 {
    1_000_000
}

pub fn read_os_timer() -> u128 {
    START_TIME.elapsed().as_micros()
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub struct Bench<const N: usize> {
    start_os: Instant,
    start_cpu: u64,
    checkpoints: [MaybeUninit<(&'static str, u64)>; N],
    cur_checkpoint: usize,
}

impl<const N: usize> Bench<N> {
    pub fn start() -> Self {
        Self {
            start_os: Instant::now(),
            start_cpu: read_cpu_timer(),
            checkpoints: [MaybeUninit::uninit(); N],
            cur_checkpoint: 0,
        }
    }
    pub fn bench(&mut self, name: &'static str) {
        debug_assert!(self.cur_checkpoint < N);
        self.checkpoints[self.cur_checkpoint].write((name, read_cpu_timer()));
        self.cur_checkpoint += 1;
    }
    pub fn end(self) {
        debug_assert!(self.cur_checkpoint == N);
        let start_cpu = self.start_cpu;
        let end_cpu = unsafe { self.checkpoints[N - 1].assume_init().1 };
        let total_time = self.start_os.elapsed().as_micros();
        let total_cpu = end_cpu - start_cpu;
        let cpu_freq: u64 = total_cpu * os_freq() / total_time as u64;
        println!(
            "Total time: {}ms (CPU freq {})",
            total_time as f64 / 1000.0,
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
