use crate::*;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token as HtmlToken, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};

pub(crate) struct HtmlSink<'a> {
    pub(crate) html_parts: &'a mut Vec<HtmlPartWithLine>,
    pub(crate) opened_elements: Vec<Element>,
    pub(crate) args: &'a Args,
}

impl<'a> TokenSink for HtmlSink<'a> {
    type Handle = ();

    fn process_token(&mut self, token: HtmlToken, line_number: u64) -> TokenSinkResult<()> {
        match token {
            HtmlToken::TagToken(tag) => match tag.kind {
                TagKind::StartTag => {
                    let mut element = Element {
                        name: tag.name.to_string(),
                        self_closing: tag.self_closing,
                        is_component: false,
                        open_attrs: tag
                            .attrs
                            .into_iter()
                            .map(|a| (a.name.local.to_string(), a.value.to_string()))
                            .collect(),
                        close_attrs: Vec::new(),
                        children: Vec::new(),
                    };

                    if element.name == "comp" || element.name == "component" {
                        let Some(real_name) = element.open_attrs.iter().find(|(k, _)| k == "name").map(|(_, v)| v) else {
                            abort!(self.args.path_span, "Missing name attribute on component tag at line {line_number}");
                        };

                        element.name = real_name.to_owned();
                        element.is_component = true;
                        element.open_attrs.retain(|(k, _)| k != "name");
                    }

                    if element.self_closing {
                        match self.opened_elements.last_mut() {
                            Some(container) => container.children.push(HtmlPartWithLine {
                                part: HtmlPart::Element(element),
                                line: line_number as usize,
                            }),
                            None => self.html_parts.push(HtmlPartWithLine {
                                part: HtmlPart::Element(element),
                                line: line_number as usize,
                            }),
                        }
                    } else {
                        self.opened_elements.push(element)
                    }
                }
                TagKind::EndTag => {
                    let mut element = self.opened_elements.pop().unwrap_or_else(|| {
                        abort!(
                            self.args.path_span,
                            "Unexpected closing tag {} at line {line_number}",
                            tag.name
                        )
                    });

                    if element.is_component {
                        if !["comp", "component"].contains(&tag.name.to_string().as_str()) {
                            abort!(
                                self.args.path_span,
                                "Unexpected closing tag {} at line {line_number}",
                                tag.name
                            );
                        }
                    } else if tag.name != element.name {
                        abort!(
                            self.args.path_span,
                            "Unexpected closing tag {} at line {line_number}",
                            tag.name
                        );
                    }

                    element.close_attrs = tag
                        .attrs
                        .into_iter()
                        .map(|a| (a.name.local.to_string(), a.value.to_string()))
                        .collect();

                    let html_part = HtmlPartWithLine {
                        part: HtmlPart::Element(element),
                        line: line_number as usize,
                    };

                    match self.opened_elements.last_mut() {
                        Some(container) => container.children.push(html_part),
                        None => self.html_parts.push(html_part),
                    }
                }
            },
            HtmlToken::CharacterTokens(text) => match self.opened_elements.last_mut() {
                Some(container) => container.children.push(HtmlPartWithLine {
                    part: HtmlPart::Text(text.to_string()),
                    line: line_number as usize,
                }),
                None => self.html_parts.push(HtmlPartWithLine {
                    part: HtmlPart::Text(text.to_string()),
                    line: line_number as usize,
                }),
            },
            HtmlToken::NullCharacterToken
            | HtmlToken::CommentToken(_)
            | HtmlToken::EOFToken
            | HtmlToken::DoctypeToken(_) => (),
            HtmlToken::ParseError(e) => abort!(
                self.args.path_span,
                format!("Failed to parse template: {e} at line {line_number}")
            ),
        }
        TokenSinkResult::Continue
    }
}

pub(crate) fn read_template(args: &Args) -> Element {
    let template = match std::fs::read_to_string(&args.path) {
        Ok(template) => template,
        Err(e) => abort!(
            args.path_span,
            "Failed to read template file at {}: {}",
            args.path,
            e
        ),
    };

    let mut html_parts = Vec::new();
    let html_sink = HtmlSink {
        html_parts: &mut html_parts,
        opened_elements: Vec::new(),
        args,
    };
    let mut html_tokenizer = Tokenizer::new(html_sink, TokenizerOpts::default());
    let mut buffer_queue = BufferQueue::new();
    buffer_queue.push_back(template.into());
    let _ = html_tokenizer.feed(&mut buffer_queue);
    html_tokenizer.end();

    let mut root = Element {
        name: "".to_string(),
        open_attrs: Vec::new(),
        close_attrs: Vec::new(),
        self_closing: false,
        is_component: false,
        children: html_parts,
    };

    root.clean_text();
    root
}
