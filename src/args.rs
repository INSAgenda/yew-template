use std::collections::HashMap;
use proc_macro::{TokenStream, TokenTree};
use crate::*;

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) path: String,
    vals: HashMap<String, TokenTree>,
}

impl Args {
    pub(crate) fn get_val(&self, id: &str, opt_required: &mut Vec<String>) -> TokenTree {
        if id.starts_with("opt_") || id.ends_with("_opt") {
            opt_required.push(id.to_string());
        }
        self.vals.get(id).map(|v| v.to_owned()).unwrap_or_else(|| panic!("Missing value for {id}"))
    }
}

pub(crate) fn parse_args(args: TokenStream) -> Args {
    let mut tokens = args.into_iter();

    // Extract the first parameter: path
    let path = match tokens.next() {
        Some(TokenTree::Literal(lit)) => {
            let path = lit.to_string();
            if !path.starts_with('"') || !path.ends_with('"') {
                panic!("First parameter should be a string literal of the path to the template file")
            }
            path[1..path.len() - 1].to_string()
        },
        Some(_) => panic!("First parameter should be a string literal of the path to the template file"),
        None => panic!("Please specify the path to the template file as the first parameter"),
    };

    let mut vals = HashMap::new();
    loop {
        // Check comma
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {},
            Some(_) => panic!("Expected a comma (or nothing) after the path to the template file"),
            None => break,
        }

        // Get ident as id
        let (id, value_if_none) = match tokens.next() {
            Some(TokenTree::Ident(ident)) => (ident.to_string(), TokenTree::Ident(ident)),
            Some(_) => panic!("Expected an identifier after the comma"),
            None => break,
        };

        // Get equal sign
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {},
            Some(_) => panic!("Expected an equal sign after the identifier"),
            None => (),
        }

        // Get value
        let value = match tokens.next() {
            Some(value) => value,
            None => value_if_none,
        };

        vals.insert(id, value);
    }

    Args { path, vals }
}
