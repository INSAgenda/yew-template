use std::collections::HashMap;
use proc_macro::{TokenStream, TokenTree, Span};
use crate::*;

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) path: String,
    pub(crate) path_span: Span,
    vals: HashMap<String, TokenTree>,
}

impl Args {
    pub(crate) fn get_val(&self, id: &str, opts: &mut Vec<String>, iters: &mut Vec<String>) -> TokenTree {
        if id.starts_with("opt_") || id.ends_with("_opt") {
            opts.push(id.to_string());
        }
        if id.starts_with("iter_") || id.ends_with("_iter") {
            iters.push(id.to_string());
        }
        self.vals.get(id).map(|v| v.to_owned()).unwrap_or_else(|| abort_call_site!("Missing value for {id}"))
    }
}

pub(crate) fn parse_args(args: TokenStream) -> Args {
    let mut tokens = args.into_iter();

    // Extract the first parameter: path
    let (path, path_span) = match tokens.next() {
        Some(TokenTree::Literal(lit)) => {
            let path = lit.to_string();
            if !path.starts_with('"') || !path.ends_with('"') {
                abort!(lit.span(), "Expected a string literal being the path to the template file");
            }
            (path[1..path.len() - 1].to_string(), lit.span())
        },
        Some(t) => abort!(t.span(), "First parameter should be a string literal of the path to the template file"),
        None => abort_call_site!("Please specify the path to the template file as the first parameter"),
    };

    let mut vals = HashMap::new();
    let mut comma_passed = false;
    loop {
        // Check comma
        if !comma_passed {
            match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {},
                Some(t) => abort!(t.span(), "Expected a comma as a separator between parameters"),
                None => break,
            }
        }
        comma_passed = false;

        // Get ident as id
        let (id, value_if_none) = match tokens.next() {
            Some(TokenTree::Ident(ident)) => (ident.to_string(), TokenTree::Ident(ident)),
            Some(t) => abort!(t.span(), "Expected an identifier after the comma"),
            None => break,
        };

        // Get equal sign
        let t = match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => punct,
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                comma_passed = true;
                vals.insert(id, value_if_none);
                continue
            },
            None => {
                vals.insert(id, value_if_none);
                break
            },
            Some(t) => abort!(t.span(), "Expected an equal sign after the identifier"),
        };

        // Get value
        let value = match tokens.next() {
            Some(value) => value,
            None => abort!(t.span(), "Expected a value after the equal sign"),
        };
        if let TokenTree::Ident(ident) = &value {
            if ident.to_string() == id {
                emit_warning!(ident.span(), "You can omit the value if it is the same as the identifier");
            }
        }

        vals.insert(id, value);
    }

    Args { path, path_span, vals }
}
