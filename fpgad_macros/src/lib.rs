#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ItemStruct, Lit, Meta, parse_macro_input, punctuated::Punctuated};

/// Procedural macro to generate platform registration code.
///
/// This macro adds a `register_platform()` method to a struct that registers it
/// with the global platform registry. The macro requires a `compat_string` parameter
/// that specifies the device tree compatibility string(s) the platform supports.
///
/// # Arguments
///
/// * `compat_string` - Comma-separated device tree compatibility strings
///
/// # Generated Code
///
/// The macro generates:
/// ```rust,ignore
/// impl YourStruct {
///     #[doc(hidden)]
///     pub fn register_platform() {
///         crate::platforms::platform::register_platform(
///             "compat_string",
///             || Box::new(Self::new())
///         );
///     }
/// }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// #[platform(compat_string = "universal")]
/// pub struct UniversalPlatform {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn platform(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the struct
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;

    // Parse the attribute arguments
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    let mut compat_string: Option<String> = None;
    for arg in args {
        if let Meta::NameValue(nv) = arg
            && nv.path.is_ident("compat_string")
            && let Expr::Lit(ref lit_expr) = nv.value
            && let Lit::Str(ref litstr) = lit_expr.lit
        {
            compat_string = Some(litstr.value());
        }
    }
    let compat_string = compat_string.expect("compat_string must be provided");

    // Generate code to register the platform
    let expanded = quote! {
        #input_struct

        impl #struct_name {
            #[doc(hidden)]
            pub fn register_platform() {
                crate::platforms::platform::register_platform(
                    #compat_string,
                    || Box::new(Self::new())
                );
            }
        }
    };
    TokenStream::from(expanded)
}
