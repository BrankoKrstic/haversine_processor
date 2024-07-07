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

use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};

fn ptr_decompose(pointer: *mut ()) -> (usize, usize, usize, usize, usize) {
    let addr = pointer as usize;
    let offset = addr & 0xFFF;
    let t1 = (addr >> 12) & 0b111111111;
    let t2 = (addr >> 21) & 0b111111111;
    let t3 = (addr >> 30) & 0b111111111;
    let t4 = (addr >> 39) & 0b111111111;
    (t4, t3, t2, t1, offset)
}

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
    let total: usize = page_size * page_count;
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
            let mut first_touched: *mut u8 = std::ptr::null_mut();
            let mut last_touched = data;
            for i in 0..touch_size {
                unsafe {
                    let cur_data = data.add(i);
                    *cur_data = i as u8;
                    last_touched = cur_data;
                    if first_touched.is_null() {
                        first_touched = cur_data;
                    }
                }
            }
            assert!(!first_touched.is_null());
            let faults_end = get_page_fault_count(process);
            let fault_count = faults_end - faults_start;
            println!(
                "{},{},{},{}",
                page_count,
                touch_count,
                fault_count,
                fault_count as usize - touch_count,
            );
            let addr = ptr_decompose(first_touched as _);
            println!(
                "FIRST TOUCHED: {},{},{},{},{}",
                addr.0, addr.1, addr.2, addr.3, addr.4
            );
            let addr2 = ptr_decompose(last_touched as _);
            println!(
                "LAST_TOUCHED: {},{},{},{},{}",
                addr2.0, addr2.1, addr2.2, addr2.3, addr2.4
            );
            unsafe { VirtualFree(data as _, 0, MEM_RELEASE).expect("Unable to free memory") };
        } else {
            eprintln!("Unable to allocate memory!");
        }
    }
    drop(handle);
}
