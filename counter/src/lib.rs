use proc_macro::TokenStream;

static mut COUNTER: usize = 0;

#[proc_macro]
pub fn counter(_item: TokenStream) -> TokenStream {
    unsafe {
        COUNTER += 1;
    }
    format!("{}", unsafe { COUNTER - 1 }).parse().unwrap()
}
