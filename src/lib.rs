//! This crate allows you to separate your HTML from your Rust code when using [Yew](https://yew.rs).
//! 
//! # Usage
//! 
//! ## Hello World
//! 
//! ```html
//! <div>
//!     <p>Hello [name]!</p>
//! </div>
//! ```
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let html = template_html!("templates/hello.html", name="World");
//! # }
//! ```
//! 
//! The code above will actually compile to the following code:
//! 
//! ```rust
//! # use yew::prelude::*;
//! # fn main() {
//! let html = html! {
//!     <div>
//!         <p>{"Hello World!"}</p>
//!     </div>
//! };
//! # }
//! ```
//! 
//! ## Using variables
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let name = "World";
//! let html = template_html!("templates/hello.html", name);
//! # }
//! ```
//! 
//! Would compile to:
//! 
//! ```rust
//! # use yew::prelude::*;
//! # fn main() {
//! let name = "World";
//! let html = html! {
//!     <div>
//!         <p>{"Hello "}{name}{"!"}</p>
//!     </div>
//! };
//! # }
//! ```
//! 
//! ## Variables with different identifiers
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let last_name = "World";
//! let html = template_html!("templates/hello.html", name=last_name);
//! # }
//! ```
//! 
//! ## Using expressions
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let name_reversed = String::from("dlroW");
//! let html = template_html!(
//!     "templates/hello.html",
//!     name = {
//!         let mut name = name_reversed.into_bytes();
//!         name.reverse();
//!         let name = String::from_utf8(name).unwrap();
//!         name
//!     }
//! );
//! # }
//! ```
//! 
//! Which will also display `Hello World!` as the output is as follows:
//! 
//! ```rust
//! # use yew::prelude::*;
//! # fn main() {
//! let name_reversed = String::from("dlroW");
//! let html = html! {
//!     <div>
//!         <p>
//!             {"Hello "}{{
//!             let mut name = name_reversed.into_bytes();
//!             name.reverse();
//!             let name = String::from_utf8(name).unwrap();
//!             name
//!             }}{"!"}
//!         </p>
//!     </div>
//! };
//! # }
//! ```
//! 
//! Note that the brackets around expressions are optional.
//! 
//! ## In attributes
//! 
//! ```html
//! <div style=[style]>
//!    <p>Hello [name]!</p>
//! </div>
//! ```
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let html = template_html!("templates/hello.html", name="World", style="color: red;");
//! # }
//! ```
//! 
//! ## Applied to Yew callbacks
//! 
//! ```html
//! <div onclick=[onclick]>
//!    <p>Hello [name]!</p>
//! </div>
//! ```
//! 
//! ```ignore
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let link = ctx.link();
//! let html = template_html!("templates/hello.html", name="World", onclick={link.callback(|_| Msg::AddOne)});
//! # }
//! ```
//! 
//! # Notes
//! 
//! - Litteral values are NOT escaped because they come from your code. Using a litteral value of `value closed by quotes" trailing stuff` will cause problems. This will be fixed in a future version.
//! 
//! - You can use multiple top-level elements in your html template file.

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

/// Reads a file and replaces the variables it contains with the supplied values. Produces a Yew html! macro invocation.
/// 
/// ```ignore
/// let html = template_html!("path", arg="value", arg2="value2", arg3={expression});
/// ```
/// 
/// See top-level documentation for more information.
#[proc_macro]
pub fn template_html(args: TokenStream) -> TokenStream {
    let args = parse_args(args);
    //println!("{args:?}");

    let code = generate_code(args);
    code.parse().unwrap()
}
