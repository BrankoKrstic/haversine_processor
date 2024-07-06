use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use proc_macro::TokenStream;

#[proc_macro]
pub fn counter(_item: TokenStream) -> TokenStream {
    let out_dir = PathBuf::from(std::env::var("COUNTER_VALUE_FOLDER").unwrap());
    let mut counter = 0;
    if let Ok(mut file) = File::open(out_dir.join("counter_val.data")) {
        let mut buf = [0; 8];
        let _ = file.read_exact(&mut buf);
        counter = u64::from_be_bytes(buf);
        counter += 1;
        drop(file);
    }

    if let Ok(mut writer) = File::create(out_dir.join("counter_val.data")) {
        writer.write_all(&counter.to_be_bytes()).unwrap();
    }

    format!("{}", counter).parse().unwrap()
}
