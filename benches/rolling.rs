use haversine_calculator::{
    bench_block,
    calc::naive_haversine,
    metrics::{record_bytes, Benchmark},
    parser::deserialize,
    CoordPair,
};
use std::{
    fs::File,
    io::{self, BufReader},
    mem::MaybeUninit,
    path::PathBuf,
};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
};

fn get_page_fault_count(process_handle: HANDLE) -> u32 {
    unsafe {
        let mut counter = MaybeUninit::uninit();
        GetProcessMemoryInfo(
            process_handle,
            counter.as_mut_ptr(),
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
        .expect("Unable to get memory info");
        counter.assume_init().PageFaultCount
    }
}

fn main() -> Result<(), io::Error> {
    let process = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            GetCurrentProcessId(),
        )
        .expect("Unable to get process handle")
    };

    loop {
        let page_faults = get_page_fault_count(process);
        bench_block!(handle, "Initial Setup");
        let mut _benchmark = Benchmark::init();
        let path = PathBuf::from("./input.json");
        let mut reader = BufReader::new(File::open(path)?);
        let mut running_sum = 0.0;
        drop(handle);
        let res: Vec<CoordPair> = deserialize(&mut reader).unwrap();
        let len = res.len();
        bench_block!(process_handle, "Process Haversine");
        for cp in res {
            record_bytes(std::mem::size_of::<CoordPair>() as u64);
            let res = naive_haversine(cp);
            running_sum += res;
        }
        let result = running_sum / len as f64;
        drop(process_handle);
        bench_block!("Print Output");
        println!("The avg is: {}", result);
        let end_page_faults = get_page_fault_count(process);
        println!("Page Faults: {}", end_page_faults - page_faults);
        println!();
        println!();
    }
}
