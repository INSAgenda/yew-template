use html5ever::{tokenizer::{TokenSink, Token as HtmlToken, TokenSinkResult, TagKind}};
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
