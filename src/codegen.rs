use html5ever::{tokenizer::{Tokenizer, TokenizerOpts, Token as HtmlToken, BufferQueue, Tag, TagKind}, Attribute};
use proc_macro::TokenTree;
use string_tools::get_all_between_strict;
use crate::*;

fn attr_to_yew_string(attr: Attribute, args: &Args) -> String {
    let name = attr.name.local.to_string();
    let value = attr.value.to_string();
    if value.starts_with('[') && value.ends_with(']') {
        let id = value[1..value.len() - 1].to_string();
        match args.vals.get(&id) {
            Some(TokenTree::Literal(lit)) => {
                let mut value = lit.to_string();
                if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                    value = value[1..value.len() - 1].to_string();
                }
                format!("{name}=\"{value}\"")
            },
            Some(TokenTree::Ident(ident)) if ident.to_string() == "true" || ident.to_string() == "false" => format!("{name}=\"{ident}\""),
            Some(value) => {
                format!("{name}={{{value}}}")
            }
            None => panic!("Missing value for {id}"),
        }
    } else {
        format!("{}=\"{}\"", name, value)
    }
}

fn html_token_to_yew_string(token: HtmlToken, args: &Args) -> String {
    match token {
        HtmlToken::TagToken(Tag { kind, name, self_closing, attrs }) => {
            let f_attrs = attrs.into_iter().map(|a| attr_to_yew_string(a, args)).collect::<Vec<_>>().join(" ");
            match kind {
                TagKind::StartTag if self_closing => format!("<{name} {f_attrs}/>"),
                TagKind::StartTag => format!("<{name} {f_attrs}>"),
                TagKind::EndTag => format!("</{name} {f_attrs}>"),
            }
        }
        HtmlToken::CharacterTokens(text) => {
            let mut text = text.to_string();
            while let Some(to_replace) = get_all_between_strict(&text, "[", "]").map(|s| s.to_string()) {
                if to_replace.chars().any(|c| !c.is_alphanumeric() && c != '_') {
                    panic!("Invalid identifier: {to_replace:?} in template {}", args.path);
                }
        
                let value = args.vals.get(&to_replace).unwrap_or_else(|| panic!("Missing value for {to_replace}"));
        
                text = text.replace(&format!("[{}]", to_replace), &format!("\"}}{{{value}}}{{\""));
            }
            text = format!("{{\"{}\"}}", text);
            text = text.replace("{\"\"}", "");

            text
        },
        HtmlToken::DoctypeToken(_) | HtmlToken::CommentToken(_) | HtmlToken::NullCharacterToken | HtmlToken::EOFToken => String::new(),
        HtmlToken::ParseError(_) => unreachable!(),
    }
}

pub(crate) fn generate_code(args: Args) -> String {
    let template = match std::fs::read_to_string(&args.path) {
        Ok(template) => template,
        Err(e) => panic!("Failed to read template file at {}: {}", args.path, e),
    };
    let mut html_tokens = Vec::new();
    let html_sink = HtmlSink { html_tokens: &mut html_tokens, current_text: String::new() };
    let mut html_tokenizer = Tokenizer::new(html_sink, TokenizerOpts::default());
    let mut buffer_queue = BufferQueue::new();
    buffer_queue.push_back(template.into());
    let _  = html_tokenizer.feed(&mut buffer_queue);
    html_tokenizer.end();
    //println!("{:#?}", html_tokens);

    let yew_html = html_tokens.into_iter().map(|t| html_token_to_yew_string(t, &args)).collect::<Vec<_>>().join("");
    let yew_code = format!("html! {{ <> {yew_html} </> }}");
    //println!("{}", yew_code);

    yew_code
}
