use crate::*;

/// Part of the template that would be considered as text by HTML
#[derive(Debug)]
pub(crate) enum TextPart {
    /// Regular text
    Literal(String),
    /// A template expression
    Expression(String),
}

impl TextPart {
    /// Parses HTML text into a list of text parts
    pub(crate) fn parse(mut s: &str, args: &Args) -> Vec<TextPart> {
        let mut parts = Vec::new();

        while let Some(text) = get_all_before_strict(s, "[") {
            s = &s[text.len() + 1..];
            if !text.is_empty() {
                parts.push(TextPart::Literal(text.to_string()));
            }
            if s.is_empty() {
                break;
            }
            let var = match get_all_before_strict(s, "]") {
                Some(var) => var,
                None => abort!(args.path_span, "Missing closing bracket in html text"),
            };
            s = &s[var.len() + 1..];
            parts.push(TextPart::Expression(var.to_string()));
        }
        if !s.is_empty() {
            parts.push(TextPart::Literal(s.to_string()));
        }

        parts
    }

    /// Turns the text part into valid Rust code for Yew
    pub(crate) fn to_code(&self, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
        text_part_to_code(self, opts, iters, args)
    }
}

pub(crate) trait HackTraitVecTextPart {
    fn to_code(&self, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String;
}

impl HackTraitVecTextPart for Vec<TextPart> {
    /// Turns a list of text parts into valid Rust code for Yew
    fn to_code(&self, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
        self.iter().map(|p| p.to_code(opts, iters, args)).collect::<Vec<_>>().join("")
    }
}
