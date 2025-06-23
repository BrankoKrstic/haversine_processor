use haversine_calculator::{
    bench_block,
    metrics::{read_cpu_timer, record_bytes, Benchmark, BenchmarkOnDrop},
};

#[link(name = "loop")]
extern "C" {
    fn Read1(count: u64, data: *const u64);
}

fn main() {
    let _benchmark = Benchmark::init();
    let mem_size = 1024 * 1024 * 1024;
    let data = (0..mem_size).collect::<Vec<_>>();
    let mut i = 0;
    let mut fract = 1024;
    while fract <= mem_size {
        let text = format!("Read bandwidth size {}", fract).leak();
        let bench = BenchmarkOnDrop::new(text, i);
        for _ in 0..(mem_size / fract) {
            unsafe { Read1(fract, data.as_ptr()) };
        }

        record_bytes(mem_size);
        drop(bench);
        fract *= 2;
        i += 1;
    }
}
