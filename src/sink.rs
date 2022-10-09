use html5ever::{tokenizer::{TokenSink, Token as HtmlToken, TokenSinkResult}};

pub(crate) struct HtmlSink<'a> {
    pub(crate) html_tokens: &'a mut Vec<HtmlToken>,
    pub(crate) current_text: String,
}

impl<'a> TokenSink for HtmlSink<'a> {
    type Handle = ();

    fn process_token(&mut self, token: HtmlToken, line_number: u64) -> TokenSinkResult<()> {
        if let HtmlToken::ParseError(e) = token {
            panic!("Failed to parse template: {e} at line {line_number}");
        }
        if let HtmlToken::CharacterTokens(new_text) = token {
            self.current_text.push_str(&new_text);
        } else {
            let mut old_text = std::mem::take(&mut self.current_text);
            old_text = old_text.trim().to_string();
            if !old_text.is_empty() {
                self.html_tokens.push(HtmlToken::CharacterTokens(old_text.into()));
            }
            self.html_tokens.push(token);
        }
        TokenSinkResult::Continue
    }
}
