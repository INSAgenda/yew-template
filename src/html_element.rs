#[derive(Debug)]
pub(crate) enum HtmlPart {
    Text(String),
    Element(Element),
}

#[derive(Debug)]
pub(crate) struct Element {
    pub(crate) name: String,
    pub(crate) self_closing: bool,
    pub(crate) open_attrs: Vec<(String, String)>,
    pub(crate) close_attrs: Vec<(String, String)>,
    pub(crate) children: Vec<HtmlPart>,
}

impl Element {
    pub(crate) fn clean_text(&mut self) {
        let mut new_children = Vec::new();
        let mut current_text = String::new();
        for child in self.children.drain(..) {
            match child {
                HtmlPart::Text(text) => {
                    current_text.push_str(&text);
                }
                HtmlPart::Element(mut element) => {
                    current_text = current_text.trim_matches(|c: char| c.is_whitespace() || c == '\n').to_string();
                    if !current_text.is_empty() {
                        new_children.push(HtmlPart::Text(current_text));
                        current_text = String::new();
                    }
                    element.clean_text();
                    new_children.push(HtmlPart::Element(element));
                }
            }
        }
        current_text = current_text.trim_matches(|c: char| c.is_whitespace() || c == '\n').to_string();
        if !current_text.is_empty() {
            new_children.push(HtmlPart::Text(current_text));
        }
        self.children = new_children;
    }
}
