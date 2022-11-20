use std::collections::HashMap;
use proc_macro::{TokenStream, TokenTree, Span, Group, Delimiter, Ident, Punct, Spacing};
use string_tools::{get_all_before, get_all_after_strict};
use crate::*;

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) path: String,
    pub(crate) path_span: Span,
    pub(crate) catalog: Catalog,
    auto_default: bool,
    vals: HashMap<String, TokenTree>,
}

impl Args {
    pub(crate) fn get_val(&self, id: &str, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> TokenTree {
        let (id, field) = (get_all_before(id, "."), get_all_after_strict(id, "."));
        if id.chars().any(|c| !c.is_alphanumeric() && c != '_') {
            abort!(args.path_span, "Invalid identifier: {id:?} in template {}", args.path);
        }
        if id.starts_with("opt_") || id.ends_with("_opt") {
            opts.push(id.to_string());
        }
        if id.starts_with("iter_") || id.ends_with("_iter") {
            iters.push(id.to_string());
        }
        let mut val: TokenTree = match self.vals.get(id).map(|v| v.to_owned()) {
            Some(val) => val,
            None if self.auto_default => {
                let val = TokenTree::Ident(Ident::new(id, Span::call_site()));
                match field {
                    Some(field) => {
                        let mut token_stream = TokenStream::new();
                        token_stream.extend(vec![val, TokenTree::Punct(Punct::new('.', Spacing::Alone)), TokenTree::Ident(Ident::new(field, args.path_span))]);
                        return TokenTree::Group(Group::new(Delimiter::Brace, token_stream))
                    }
                    None => val
                }
            }
            None => abort_call_site!(format!("Missing value for {id}")),
        };
        if let Some(field) = field {
            let mut token_stream = TokenStream::new();
            token_stream.extend(vec![val, TokenTree::Punct(Punct::new('.', Spacing::Alone)), TokenTree::Ident(Ident::new(field, args.path_span))]);
            val = TokenTree::Group(Group::new(Delimiter::Brace, token_stream));
        }
        val
    }
}

pub(crate) fn parse_args(args: TokenStream) -> Args {
    let config = config::read_config();
    
    let mut tokens = args.into_iter().peekable();
    let _ = tokens.peek();

    // Extract the first parameter: path
    let (path, path_span) = match tokens.next() {
        Some(TokenTree::Literal(lit)) => {
            let path = lit.to_string();
            if !path.starts_with('"') || !path.ends_with('"') {
                abort!(lit.span(), "Expected a string literal being the path to the template file");
            }
            let path = path[1..path.len() - 1].to_string();
            (format!("{}{}", config.template_folder, path), lit.span())
        },
        Some(t) => abort!(t.span(), "First parameter should be a string literal of the path to the template file"),
        None => abort_call_site!("Please specify the path to the template file as the first parameter"),
    };

    let mut vals = HashMap::new();
    let mut comma_passed = false;
    let mut auto_default = config.auto_default;
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

        // Get ident as id, or close the group and enable auto-default
        let (id, value_if_none) = match tokens.next() {
            Some(TokenTree::Ident(ident)) => (ident.to_string(), TokenTree::Ident(ident)),
            Some(TokenTree::Punct(punct)) if punct.as_char() == '.' && punct.spacing() == Spacing::Joint => {
                match tokens.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                        match tokens.next() {
                            Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                                match tokens.next() {
                                    Some(_) => abort!(punct.span(), "Dots should be at the end of the parameter list and nothing should follow"),
                                    None => {
                                        auto_default = true;
                                        break;
                                    }
                                }
                            }
                            Some(_) => abort!(punct.span(), "Expected a third dot for enabling auto-default"),
                            None => {
                                auto_default = true;
                                break;
                            }
                        }
                    }
                    _ => abort!(punct.span(), "Expected a second dot for enabling auto-default"),
                }
            }
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

    // TODO
    let catalog = Catalog::new(&["fr"], "en");

    Args { path, path_span, vals, auto_default, catalog }
}
