#![allow(clippy::unit_arg)]

use proc_macro::TokenStream;
use syn::{ItemMod, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn sysfs_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("item: {:?}", parse_macro_input!(item as ItemMod));

    TokenStream::new()
}
