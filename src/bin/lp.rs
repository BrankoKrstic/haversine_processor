use haversine_calculator::metrics::read_cpu_timer;

#[link(name = "loop")]
extern "C" {
    fn Write1(count: u64, data: *mut u64);
    fn Write2(count: u64, data: *mut u64);
    fn Write3(count: u64, data: *mut u64);
    fn Write4(count: u64, data: *mut u64);
    fn Write5(count: u64, data: *mut u64);
}

fn main() {
    let count = 10_000_000;
    let start = std::time::Instant::now();
    // unsafe { MOVAllBytesASM(count, &mut data as *mut _) };
    // unsafe { NOPAllBytesASM(count) };
    // unsafe { CMPAllBytesASM(count) };
    let mut data = 125u64;
    unsafe { Write5(count, &mut data as *mut u64) };

    let elapsed = start.elapsed().as_micros() as f64 / 1_000_000.0;
    let throughput = count as f64 / elapsed / (1_000_000_000.0);
    println!("Executed {} Gb/s", throughput);
    // let throughput = count / elapsed / (1_000_000_000);
    // println!("Executed {}Gb/s", throughput);

    // let start = std::time::Instant::now();
    // unsafe { NOPAllBytesASM(count) };
    // let elapsed = start.elapsed().as_secs();
    // let throughput = count / elapsed / (1_000_000_000);
    // println!("Executed {}Gb/s", throughput);

    // let start = std::time::Instant::now();
    // unsafe { CMPAllBytesASM(count) };
    // let elapsed = start.elapsed().as_micros() as f64 / 1_000_000.0;
    // let throughput = count as f64 / elapsed / (1_000_000_000.0);
    // println!("Executed {}Gb/s", throughput);

    // let start = std::time::Instant::now();
    // unsafe { DECAllBytesASM(count) };
    // let elapsed = start.elapsed().as_secs();
    // let throughput = count / elapsed / (1_000_000_000);
    // println!("Executed {}Gb/s", throughput);
}
