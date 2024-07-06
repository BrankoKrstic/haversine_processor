use std::path::PathBuf;

fn main() {
    println!(
        "cargo:rustc-env=COUNTER_VALUE_FOLDER={}",
        std::env::var("OUT_DIR").unwrap()
    );
    println!("HEY");
    let _ = std::fs::remove_file(
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("counter_val.data"),
    );
}
