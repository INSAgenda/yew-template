use html5ever::tokenizer::{Tokenizer, TokenizerOpts, BufferQueue};
use proc_macro::TokenTree;
use string_tools::get_all_between_strict;
use crate::*;

fn attr_to_yew_string((name, value): (String, String), opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    if name == "opt" || name == "iter" {
        return String::new()
    }
    if value.starts_with('[') && value.ends_with(']') && !value[1..value.len() - 1].chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
        let id = value[1..value.len() - 1].to_string();
        match args.get_val(&id, opts, iters) {
            TokenTree::Literal(lit) => {
                let mut value = lit.to_string();
                if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                    value = value[1..value.len() - 1].to_string();
                }
                format!("{name}=\"{value}\"")
            },
            TokenTree::Ident(ident) if ident.to_string() == "true" || ident.to_string() == "false" => format!("{name}=\"{ident}\""),
            value => {
                format!("{name}={{{value}}}")
            }
        }
    } else if value.contains('[') && value.contains(']') {
        let mut text = value;
        let mut end = Vec::new();
        while let Some(to_replace) = get_all_between_strict(&text, "[", "]").map(|s| s.to_string()) {
            if to_replace.chars().any(|c| !c.is_alphanumeric() && c != '_') {
                panic!("Invalid identifier: {to_replace:?} in template {}", args.path);
            }
    
            let mut value = args.get_val(&to_replace, opts, iters).to_string();
            if to_replace.starts_with("opt_") || to_replace.ends_with("_opt") || to_replace.starts_with("iter_") || to_replace.ends_with("_iter") {
                value = format!("macro_produced_{to_replace}");
            };
    
            if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                value = value[1..value.len() - 1].to_string();
                text = text.replace(&format!("[{}]", to_replace), &value);
            } else {
                text = text.replace(&format!("[{}]", to_replace), "{}");
                end.push(value);
            }
        }
        match end.is_empty() {
            true => format!("{name}=\"{text}\""),
            false => format!("{name}={{ format!(\"{text}\", {}) }}", end.join(", "))
        }
    } else {
        format!("{}=\"{}\"", name, value)
    }
}

fn html_part_to_yew_string(part: HtmlPart, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    match part {
        HtmlPart::Element(element) => {
            if element.self_closing && !element.close_attrs.is_empty() {
                panic!("Self-closing tags cannot have closing attributes");
            }
            if element.self_closing && !element.children.is_empty() {
                panic!("Self-closing tags cannot have children");
            }
            let opt = element.open_attrs.iter().any(|(n,_)| n=="opt");
            let iter = element.open_attrs.iter().any(|(n,_)| n=="iter");

            let mut inner_opts = Vec::new();
            let mut inner_iters = Vec::new();
            let f_open_attrs = element.open_attrs.into_iter().map(|a| attr_to_yew_string(a, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join(" ");
            let f_close_attrs = element.close_attrs.into_iter().map(|a| attr_to_yew_string(a, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join(" ");
            let name = element.name;
            let mut content = element.children.into_iter().map(|p| html_part_to_yew_string(p, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join("");
            inner_opts.sort();
            inner_opts.dedup();
            inner_iters.sort();
            inner_iters.dedup();

            match opt {
                true => {
                    let left = inner_opts.iter().map(|id| format!("Some(macro_produced_{id})")).collect::<Vec<_>>().join(", ");
                    let right = inner_opts.iter().map(|id| args.get_val(id, &mut Vec::new(), &mut Vec::new()).to_string()).collect::<Vec<_>>().join(", ");
                    content = format!("if let ({left}) = ({right}) {{ {content} }}");
                },
                false => opts.extend_from_slice(&inner_opts),
            }

            match iter {
                true => {
                    let before = inner_iters
                        .iter()
                        .map(|id| format!("let mut macro_produced_{id} = {};", args.get_val(id, &mut Vec::new(), &mut Vec::new())))
                        .collect::<Vec<_>>()
                        .join("");
                    let left = inner_iters.iter().map(|id| format!("Some(macro_produced_{id})")).collect::<Vec<_>>().join(", ");
                    let right = inner_iters.iter().map(|id| format!("macro_produced_{id}.next()", )).collect::<Vec<_>>().join(", ");
                    content = format!("{{{{
                        {before}
                        let mut fragments = Vec::new();
                        while let ({left}) = ({right}) {{
                            fragments.push(html! {{ <> {content} </> }});
                        }}
                        fragments.into_iter().collect::<yew::Html>()
                    }}}}");
                },
                false => iters.extend_from_slice(&inner_iters),
            }

            match element.self_closing {
                true => format!("<{name} {f_open_attrs}/>"),
                false => format!("<{name} {f_open_attrs}>{content}</{name} {f_close_attrs}>\n"),
            }
        }
        HtmlPart::Text(mut text) => {
            while let Some(to_replace) = get_all_between_strict(&text, "[", "]").map(|s| s.to_string()) {
                if to_replace.chars().any(|c| !c.is_alphanumeric() && c != '_') {
                    panic!("Invalid identifier: {to_replace:?} in template {}", args.path);
                }
        
                let mut value = args.get_val(&to_replace, opts, iters).to_string();
                if to_replace.starts_with("opt_") || to_replace.ends_with("_opt") || to_replace.starts_with("iter_") || to_replace.ends_with("_iter") {
                    value = format!("macro_produced_{to_replace}");
                };
        
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

    let yew_html = html_part_to_yew_string(HtmlPart::Element(root), &mut Vec::new(), &mut Vec::new(), &args);
    let yew_code = format!("html! {{ {yew_html} }}");
    println!("{}", yew_code);

    yew_code
}
