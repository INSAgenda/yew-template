use html5ever::{tokenizer::{TokenSink, Token as HtmlToken, TokenSinkResult, TagKind, Tokenizer, TokenizerOpts, BufferQueue}};
use crate::*;

pub(crate) struct HtmlSink<'a> {
    pub(crate) html_parts: &'a mut Vec<HtmlPart>,
    pub(crate) opened_elements: Vec<Element>,
    pub(crate) args: &'a Args,
}

impl<'a> TokenSink for HtmlSink<'a> {
    type Handle = ();

    fn process_token(&mut self, token: HtmlToken, line_number: u64) -> TokenSinkResult<()> {
        match token {
            HtmlToken::TagToken(tag) => match tag.kind {
                TagKind::StartTag => {
                    let element = Element {
                        name: tag.name.to_string(),
                        self_closing: tag.self_closing,
                        open_attrs: tag.attrs.into_iter().map(|a| (a.name.local.to_string(), a.value.to_string())).collect(),
                        close_attrs: Vec::new(),
                        children: Vec::new(),
                    };
                    match element.self_closing {
                        true => match self.opened_elements.last_mut() {
                            Some(container) => container.children.push(HtmlPart::Element(element)),
                            None => self.html_parts.push(HtmlPart::Element(element)),
                        },
                        false => self.opened_elements.push(element)
                    }
                },
                TagKind::EndTag => {
                    let mut element = self.opened_elements.pop().unwrap_or_else(|| abort!(self.args.path_span, "Unexpected closing tag {} at line {line_number}", tag.name));
                    if tag.name != element.name {
                        abort!(self.args.path_span, "Unexpected closing tag {} at line {line_number}", tag.name);
                    }
                    element.close_attrs = tag.attrs.into_iter().map(|a| (a.name.local.to_string(), a.value.to_string())).collect();
                    match self.opened_elements.last_mut() {
                        Some(container) => container.children.push(HtmlPart::Element(element)),
                        None => self.html_parts.push(HtmlPart::Element(element)),
                    }
                },
            },
            HtmlToken::CharacterTokens(text) => match self.opened_elements.last_mut() {
                Some(container) => container.children.push(HtmlPart::Text(text.to_string())),
                None => self.html_parts.push(HtmlPart::Text(text.to_string())),
            },
            HtmlToken::NullCharacterToken | HtmlToken::CommentToken(_) | HtmlToken::EOFToken | HtmlToken::DoctypeToken(_) => (),
            HtmlToken::ParseError(e) => abort!(self.args.path_span, format!("Failed to parse template: {e} at line {line_number}")),
        }
        TokenSinkResult::Continue
    }
}

pub(crate) fn read_template(args: &Args) -> Element {
    let template = match std::fs::read_to_string(&args.path) {
        Ok(template) => template,
        Err(e) => abort!(args.path_span, "Failed to read template file at {}: {}", args.path, e),
    };
    let mut html_parts = Vec::new();
    let html_sink = HtmlSink { html_parts: &mut html_parts, opened_elements: Vec::new(), args };
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
    root
}
