use std::mem::MaybeUninit;

use haversine_calculator::bench_block;
use haversine_calculator::metrics::Benchmark;
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
};

use windows::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};

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

fn main() {
    let process = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            GetCurrentProcessId(),
        )
        .expect("Unable to get process handle")
    };
    let args: Vec<String> = std::env::args().collect();

    let page_size = 4096usize;
    let page_count: usize = args[1].parse().unwrap();
    let total = page_size * page_count;
    let mut _benchmark = Benchmark::init();
    bench_block!(handle, "Initial Setup");
    // println!("Page Count,Touch Count,Fault Count,Extra Faults");
    for touch_count in 0..page_count {
        let touch_size = touch_count * page_size;
        let data = unsafe {
            VirtualAlloc(None, total, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) as *mut u8
        };
        if !data.is_null() {
            let faults_start = get_page_fault_count(process);

            for i in 0..touch_size {
                unsafe {
                    *data.add(i) = i as u8;
                }
            }
            let faults_end = get_page_fault_count(process);
            let fault_count = faults_end - faults_start;
            println!(
                "{},{},{},{}",
                page_count,
                touch_count,
                fault_count,
                fault_count as usize - touch_count
            )
        } else {
            eprintln!("Unable to allocate memory!");
        }
    }
    drop(handle);
}
