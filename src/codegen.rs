use proc_macro::TokenTree;
use crate::*;

/// Turns an HTML attribute to Rust code for Yew
fn attr_to_code((name, value): (String, String), opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> Option<String> {
    // Remove attributes used by yew-template
    if name == "opt" || name == "iter" || name == "present-if" {
        return None
    }
    Some(if value.starts_with('[') && value.ends_with(']') && !value[1..value.len() - 1].chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
        let id = value[1..value.len() - 1].to_string();
        match args.get_val(&id, opts, iters, args) {
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
        let mut i = 0;
        while let Some(to_replace) = get_all_between_strict(&text, "[", "]").map(|s| s.to_string()) {
            let mut value = args.get_val(&to_replace, opts, iters, args).to_string();
            if to_replace.starts_with("opt_") || to_replace.ends_with("_opt") || to_replace.starts_with("iter_") || to_replace.ends_with("_iter") {
                value = format!("macro_produced_{to_replace}");
            };
    
            if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                value = value[1..value.len() - 1].to_string();
                text = text.replace(&format!("[{}]", to_replace), &value);
            } else {
                text = text.replace(&format!("[{}]", to_replace), &format!("{{attribute_value_{i}}}"));
                end.push(format!("attribute_value_{i} = {value}"));
                i += 1;
            }
        }
        match end.is_empty() {
            true => format!("{name}=\"{text}\""),
            false => format!("{name}={{ format!(\"{text}\", {}) }}", end.join(", "))
        }
    } else {
        format!("{}=\"{}\"", name, value)
    })
}

/// Turns an HTML element and its children to Rust code for Yew
pub(crate) fn element_to_code(el: Element, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let tabs = "    ".repeat(depth);

    // Make sure the element is valid
    if el.self_closing && !el.close_attrs.is_empty() {
        abort!(args.path_span, "Self-closing tags cannot have closing attributes");
    }
    if el.self_closing && !el.children.is_empty() {
        abort!(args.path_span, "Self-closing tags cannot have children");
    }

    // Get element properties for templating
    let opt = el.open_attrs.iter().any(|(n,_)| n=="opt");
    let iter = el.open_attrs.iter().any(|(n,_)| n=="iter");
    let present_if = el.open_attrs.iter().find(|(n,_)| n=="present-if").map(|(_,v)| v.to_owned());

    // Scan and generate children
    let mut inner_opts = Vec::new();
    let mut inner_iters = Vec::new();
    let mut f_open_attrs = el.open_attrs.into_iter().filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join(" ");
    if !f_open_attrs.is_empty() {
        f_open_attrs.insert(0, ' ');
    }
    let mut f_close_attrs = el.close_attrs.into_iter().filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join(" ");
    if !f_close_attrs.is_empty() {
        f_close_attrs.insert(0, ' ');
    }
    let name = el.name;
    let mut content = el.children.into_iter().map(|p| p.part.into_code(depth + 1, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join("");
    inner_opts.sort();
    inner_opts.dedup();
    inner_iters.sort();
    inner_iters.dedup();

    // Handle special virtual elements
    content = match name == "virtual" {
        true => {
            if !f_open_attrs.is_empty() || !f_close_attrs.is_empty() {
                abort!(args.path_span, "Virtual elements cannot have attributes (found {:?} and {:?})", f_open_attrs, f_close_attrs);
            }
            content.replace("\n    ", "\n")
        },
        false => match el.self_closing {
            true if &name == "br" => format!("<{name} {f_open_attrs}/>"),
            true => format!("\n{tabs}<{name}{f_open_attrs}/>"),
            false => format!("\n{tabs}<{name}{f_open_attrs}>{content}\n{tabs}</{name}{f_close_attrs}>"),
        }
    };

    // Handle optional elements
    match opt {
        true => {
            let left = inner_opts.iter().map(|id| format!("Some(macro_produced_{id})")).collect::<Vec<_>>().join(", ");
            let right = inner_opts.iter().map(|id| args.get_val(id, &mut Vec::new(), &mut Vec::new(), args).to_string()).collect::<Vec<_>>().join(", ");
            content = content.replace('\n', "\n    ");
            content = format!("\n{tabs}if let ({left}) = ({right}) {{ {content}\n{tabs}}}");
        },
        false => opts.extend_from_slice(&inner_opts),
    }

    // Handle iterated elements
    match iter {
        true => {
            let before = inner_iters
                .iter()
                .map(|id| format!("let mut macro_produced_{id} = {};", args.get_val(id, &mut Vec::new(), &mut Vec::new(), args)))
                .collect::<Vec<_>>()
                .join("");
            let left = inner_iters.iter().map(|id| format!("Some(macro_produced_{id})")).collect::<Vec<_>>().join(", ");
            let right = inner_iters.iter().map(|id| format!("macro_produced_{id}.next()", )).collect::<Vec<_>>().join(", ");
            content = content.replace('\n', "\n        ");
            content = format!("\n\
                {tabs}{{{{\n\
                {tabs}{before}\n\
                {tabs}let mut fragments = Vec::new();\n\
                {tabs}while let ({left}) = ({right}) {{\n\
                {tabs}    fragments.push(yew::html! {{ <> {content} \n\
                {tabs}    </> }});\n\
                {tabs}}}\n\
                {tabs}fragments.into_iter().collect::<yew::Html>()\n\
                {tabs}}}}}"
            );
        },
        false => iters.extend_from_slice(&inner_iters),
    }

    // Handle optionaly present elements
    if let Some(mut present_if) = present_if {
        let not = present_if.starts_with('!');
        if not {
            present_if = present_if[1..].to_string();
        }
        let negation = if not {"!"} else {""};
        if !present_if.starts_with('[') || !present_if.ends_with(']') {
            abort!(args.path_span, "present_if attribute must be a variable");
        }
        present_if = present_if[1..present_if.len()-1].to_string();
        let val = args.get_val(&present_if, &mut Vec::new(), &mut Vec::new(), args);
        content = content.replace('\n', "\n    ");
        content = format!("\n\
            {tabs}if {negation}{{{val}}} {{\
            {tabs}{content}\n\
            {tabs}}}"
        );
    }

    content
}

/// Turns HTML text to Rust code for Yew
pub(crate) fn text_to_code(text: String, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let tabs = "\t".repeat(depth);

    // If it's only a single variable then no need to translate
    let text_parts = TextPart::parse(&text, args);
    if matches!(text_parts.as_slice(), &[TextPart::Expression(_)]) {
        return text_parts[0].to_code(opts, iters, args);
    }

    // Get localized texts
    #[cfg(feature = "i18n")]
    let translations = args.catalog.translate_text(&text, args);
    #[cfg(not(feature = "i18n"))]
    let translations = vec![(String::new(), text_parts)];

    // Translations are disabled
    if translations.len() == 1 {
        return format!("\n{tabs}{}", translations[0].1.to_code(opts, iters, args));
    }

    // It's a simple case with a static string
    let mut all_are_single_literal = true;
    for (_, translation) in &translations {
        if translation.len() != 1 || !matches!(translation[0], TextPart::Literal(_)) {
            all_are_single_literal = false;
            break;
        }
    }
    let locale_code = &args.config.locale_code;
    if all_are_single_literal {
        let mut result = String::new();
        result.push_str(&format!("\n{tabs}{{match {locale_code} {{\n"));
        for (i, (locale, translation)) in translations.iter().enumerate().rev() {
            let arm = match i == 0 {
                true => String::from("_"),
                false => format!("\"{locale}\""),
            };
            let text = match &translation[0] {
                TextPart::Literal(l) => l,
                _ => unreachable!(),
            };
            result.push_str(&format!("{tabs}    {arm} => \"{text}\",\n"));
        }
        result.push_str(&format!("{tabs}}}}}"));
        return result;
    }

    // It's a complex case
    let mut result = String::new();
    result.push_str(&format!("\n{tabs}{{match {locale_code} {{\n"));
    for (i, (locale, translation)) in translations.iter().enumerate().rev() {
        let arm = match i == 0 {
            true => String::from("_"),
            false => format!("\"{locale}\""),
        };
        let code = translation.to_code(opts, iters, args);
        result.push_str(&format!("{tabs}    {arm} => yew::html! {{ <> {code} </> }},\n"));
    }
    result.push_str(&format!("{tabs}}}}}"));

    result
}

pub(crate) fn generate_code(root: Element, args: Args) -> String {
    //println!("{:#?}", root);

    let yew_html = HtmlPart::Element(root).into_code(0, &mut Vec::new(), &mut Vec::new(), &args);
    let yew_code = format!("yew::html! {{ {yew_html} }}");
    println!("{}", yew_code);

    yew_code
}
