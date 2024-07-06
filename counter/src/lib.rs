use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use proc_macro::TokenStream;

// Experimental way I found to recreate the C __COUNTER__ preprocessor directive
// Currently used to record performance of a specific code block to its respective slot in a buffer without runtime overhead of a table lookup
// Macro relies on the existence of a build script which deletes the the temp file between compilation runs
// It will likely fail miserably with incremental builds, and side-effects are not recommended inside proc macros, so this shouldn't be used anywhere else
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
