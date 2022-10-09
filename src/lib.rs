extern crate proc_macro;
use std::collections::HashMap;

use html5ever::{tokenizer::{Tokenizer, TokenizerOpts, TokenSink, Token as HtmlToken, BufferQueue, TokenSinkResult, Doctype, Tag, TagKind}, buffer_queue, Attribute};
use proc_macro::{TokenStream, TokenTree};
use string_tools::get_all_between_strict;

#[derive(Debug)]
struct Args {
    path: String,
    vals: HashMap<String, TokenTree>,
}

fn parse_args(args: TokenStream) -> Args {
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

struct HtmlSink<'a> {
    html_tokens: &'a mut Vec<HtmlToken>,
    current_text: String,
}

impl<'a> TokenSink for HtmlSink<'a> {
    type Handle = ();

    fn process_token(&mut self, token: HtmlToken, line_number: u64) -> TokenSinkResult<()> {
        if let HtmlToken::ParseError(e) = token {
            panic!("Failed to parse template: {e} at line {line_number}");
        }
        if let HtmlToken::CharacterTokens(new_text) = token {
            self.current_text.push_str(&new_text);
        } else {
            let mut old_text = std::mem::take(&mut self.current_text);
            old_text = old_text.trim().to_string();
            if !old_text.is_empty() {
                self.html_tokens.push(HtmlToken::CharacterTokens(old_text.into()));
            }
            self.html_tokens.push(token);
        }
        TokenSinkResult::Continue
    }
}

fn attr_to_yew_string(attr: Attribute) -> String {
    format!("{}=\"{}\"", attr.name.local, attr.value) // TODO
}

fn html_token_to_yew_string(token: HtmlToken) -> String {
    match token {
        HtmlToken::TagToken(Tag { kind, name, self_closing, attrs }) => {
            let f_attrs = attrs.into_iter().map(attr_to_yew_string).collect::<Vec<_>>().join(" ");
            match kind {
                TagKind::StartTag if self_closing => format!("<{name} {f_attrs}/>"),
                TagKind::StartTag => format!("<{name} {f_attrs}>"),
                TagKind::EndTag => format!("</{name} {f_attrs}>"),
            }
        }
        HtmlToken::CharacterTokens(text) => format!("{{\"{}\"}}", text),
        HtmlToken::DoctypeToken(_) | HtmlToken::CommentToken(_) | HtmlToken::NullCharacterToken | HtmlToken::EOFToken => String::new(),
        HtmlToken::ParseError(_) => unreachable!(),
    }
}

fn generate_code(args: Args) -> String {
    let mut template = match std::fs::read_to_string(&args.path) {
        Ok(template) => template,
        Err(e) => panic!("Failed to read template file at {}: {}", args.path, e),
    };
    let mut html_tokens = Vec::new();
    let html_sink = HtmlSink { html_tokens: &mut html_tokens, current_text: String::new() };
    let mut html_tokenizer = Tokenizer::new(html_sink, TokenizerOpts::default());
    let mut buffer_queue = BufferQueue::new();
    buffer_queue.push_back(template.clone().into());
    let _  = html_tokenizer.feed(&mut buffer_queue);
    html_tokenizer.end();

    println!("{:#?}", html_tokens);
    let yew_text = html_tokens.into_iter().map(html_token_to_yew_string).collect::<Vec<_>>().join("");
    println!("{}", yew_text);

    while let Some(to_replace) = get_all_between_strict(&template, "{", "}").map(|s| s.to_string()) {
        if to_replace.chars().any(|c| !c.is_alphanumeric() && c != '_') {
            panic!("Invalid identifier: {to_replace:?} in template {}", args.path);
        }

        let value = args.vals.get(&to_replace).unwrap_or_else(|| panic!("Missing value for {to_replace}"));

        template = template.replace(&format!("{{{}}}", to_replace), &value.to_string());
    }

    template
}

#[proc_macro]
pub fn template_html(args: TokenStream) -> TokenStream {
    let args = parse_args(args);
    println!("{args:?}");

    let template = generate_code(args);
    //let template = "";
    println!("{template:?}");

    let code = format!("const CODE: &str = r#\"{}\"#;", template);
    code.parse().unwrap()
}
