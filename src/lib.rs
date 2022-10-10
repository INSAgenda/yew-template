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
//! Note that the brackets around expressions are required for expressions.
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
//! ## Optional variables
//! 
//! Optional variables are marked with an `opt_` prefix or an `_opt` suffix, at your option.
//! Their value is expected to be an `Option<T>`.
//! 
//! Optional variables work with optional html elements. Mark an element with the `opt` attribute to make it optional. An optional element will only be rendered if *ALL* the optional variables it contains are `Some`. Note that variables contained by smaller optional elements are excluded from this requirement.
//! 
//! ```html
//! <div>
//!     <p>Hello [name]!</p>
//!     <div opt>
//!         <h2>Age</h2>
//!         <p>You are [opt_age] years old!</p>
//!     </div>
//! </div>
//! ```
//! 
//! In the example above, the `div` block will not be shown if `opt_age` is `None`.
//! 
//! Let's see how optional elements can be nested.
//! 
//! ```html
//! <div>
//!     <p>Hello [name]!</p>
//!     <div opt>
//!         <h2>Age</h2>
//!         <p>You are [opt_age] years old!</p>
//!         <p opt>And you are born in [opt_birth_city].</p>
//!     </div>
//! </div>
//! ```
//! 
//! Here, both `opt_age` and `opt_birth_city` are optional. `opt_age` would be displayed even if `opt_birth_city` is `None`. However, if `opt_age` is `None`, `opt_birth_city` will not be displayed regardless of its value.
//! 
//! From the Rust side, there is no usage difference. Note that brackets are required (for now).
//! 
//! ```rust
//! # use yew_template::*;
//! # use yew::prelude::*;
//! # fn main() {
//! let opt_age: Option<u8> = Some(20);
//! let opt_birth_city: Option<String> = None;
//! let html = template_html!("templates/opt.html", name="John", opt_age, opt_birth_city);
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
