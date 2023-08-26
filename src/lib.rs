#![doc = include_str!("../README.md")]

extern crate proc_macro;
use proc_macro::TokenStream;

mod args;
mod codegen;
mod config;
mod helper;
mod html_element;
#[cfg(feature = "i18n")]
mod i18n;
mod sink;
mod text_part;
#[cfg(feature = "i18n")]
pub(crate) use crate::i18n::*;
pub(crate) use {
    crate::args::*, crate::codegen::*, crate::config::*, crate::helper::*, crate::html_element::*,
    crate::sink::*, crate::text_part::*, proc_macro_error::*, std::collections::HashMap,
    string_tools::*,
};

/// Reads a file and replaces the variables it contains with the supplied values. Produces a Yew html! macro invocation.
///
/// ```rust ignore
/// let html = yew_template::template_html!("path", arg="value", arg2="value2", arg3={expression});
/// ```
///
/// See top-level documentation for more information.
#[proc_macro]
#[proc_macro_error]
pub fn template_html(args: TokenStream) -> TokenStream {
    let args = parse_args(args);
    let root = read_template(&args);
    #[cfg(feature = "i18n")]
    generate_pot(&root, &args);
    let code = generate_code(root, args);
    #[cfg(feature = "log")]
    println!("{code}");
    code.parse().unwrap()
}
