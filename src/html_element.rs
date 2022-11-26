use crate::*;

#[derive(Debug)]
pub(crate) enum HtmlPart {
    Text(String),
    Element(Element),
}

impl HtmlPart {
    /// Turn the HTML part into Rust code for Yew
    pub(crate) fn into_code(self, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
        match self {
            HtmlPart::Element(el) => element_to_code(el, depth, opts, iters, args),
            HtmlPart::Text(text) => text_to_code(text, depth, opts, iters, args),
        }
    }
}

#[derive(Debug)]
pub(crate) struct HtmlPartWithLine {
    pub(crate) part: HtmlPart,
    pub(crate) line: usize,
}

#[derive(Debug)]
pub(crate) struct Element {
    pub(crate) name: String,
    pub(crate) self_closing: bool,
    pub(crate) open_attrs: Vec<(String, String)>,
    pub(crate) close_attrs: Vec<(String, String)>,
    pub(crate) children: Vec<HtmlPartWithLine>,
}

impl Element {
    pub(crate) fn clean_text(&mut self) {
        let mut new_children = Vec::new();
        let mut current_text = String::new();
        let mut current_line = None;
        for child in self.children.drain(..) {
            match child.part {
                HtmlPart::Text(text) => {
                    current_text.push_str(&text);
                    if current_line.is_none() {
                        current_line = Some(child.line);
                    }
                }
                HtmlPart::Element(mut element) => {
                    current_text = current_text.trim_matches(|c: char| (c.is_whitespace() || c == '\n') && c != '\u{A0}').to_string();
                    if !current_text.is_empty() {
                        new_children.push(HtmlPartWithLine {
                            part: HtmlPart::Text(current_text),
                            line: current_line.unwrap(),
                        });
                        current_text = String::new();
                        current_line = None;
                    }
                    element.clean_text();
                    new_children.push(HtmlPartWithLine { part: HtmlPart::Element(element), line: child.line });
                }
            }
        }
        current_text = current_text.trim_matches(|c: char| (c.is_whitespace() || c == '\n') && c != '\u{A0}').to_string();
        if !current_text.is_empty() {
            new_children.push(HtmlPartWithLine {
                part: HtmlPart::Text(current_text),
                line: current_line.unwrap(),
            });
        }
        self.children = new_children;
    }
}
