use proc_macro::TokenStream;

extern crate quote;
extern crate syn;

#[proc_macro_attribute]
pub fn audited(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
