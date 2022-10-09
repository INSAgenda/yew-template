extern crate proc_macro;
use proc_macro::TokenStream;

mod args;
mod codegen;
mod sink;
pub(crate) use {
    crate::args::*,
    crate::codegen::*,
    crate::sink::*,
};

#[proc_macro]
pub fn template_html(args: TokenStream) -> TokenStream {
    let args = parse_args(args);
    println!("{args:?}");

    let code = generate_code(args);
    code.parse().unwrap()
}
