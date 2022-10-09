use html5ever::{tokenizer::{Tokenizer, TokenizerOpts, Token as HtmlToken, BufferQueue, Tag, TagKind}, Attribute};
use proc_macro::TokenTree;
use string_tools::get_all_between_strict;
use crate::*;

fn attr_to_yew_string((name, value): (String, String), args: &Args) -> String {
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

fn html_part_to_yew_string(part: HtmlPart, args: &Args) -> String {
    match part {
        HtmlPart::Element(element) => {
            if element.self_closing && !element.close_attrs.is_empty() {
                panic!("Self-closing tags cannot have closing attributes");
            }
            if element.self_closing && !element.children.is_empty() {
                panic!("Self-closing tags cannot have children");
            }

            let f_open_attrs = element.open_attrs.into_iter().map(|a| attr_to_yew_string(a, args)).collect::<Vec<_>>().join(" ");
            let f_close_attrs = element.close_attrs.into_iter().map(|a| attr_to_yew_string(a, args)).collect::<Vec<_>>().join(" ");
            let name = element.name;
            let content = element.children.into_iter().map(|p| html_part_to_yew_string(p, args)).collect::<Vec<_>>().join("");

            match element.self_closing {
                true => format!("<{name} {f_open_attrs}/>"),
                false => format!("<{name} {f_open_attrs}>{content}</{name} {f_close_attrs}>"),
            }
        }
        HtmlPart::Text(mut text) => {
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
        }
    }
}

pub(crate) fn generate_code(args: Args) -> String {
    let template = match std::fs::read_to_string(&args.path) {
        Ok(template) => template,
        Err(e) => panic!("Failed to read template file at {}: {}", args.path, e),
    };
    let mut html_parts = Vec::new();
    let html_sink = HtmlSink { html_parts: &mut html_parts, opened_elements: Vec::new() };
    let mut html_tokenizer = Tokenizer::new(html_sink, TokenizerOpts::default());
    let mut buffer_queue = BufferQueue::new();
    buffer_queue.push_back(template.into());
    let _  = html_tokenizer.feed(&mut buffer_queue);
    html_tokenizer.end();
    let mut root = Element {
        name: "".to_string(),
        open_attrs: Vec::new(),
        close_attrs: Vec::new(),
        self_closing: false,
        children: html_parts,
    };
    root.clean_text();
    println!("{:#?}", root);

    let yew_html = html_part_to_yew_string(HtmlPart::Element(root), &args);
    let yew_code = format!("html! {{ {yew_html} }}");
    println!("{}", yew_code);

    yew_code
}
