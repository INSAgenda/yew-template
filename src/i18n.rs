use std::collections::HashMap;
use poreader::{PoParser, Message};
use string_tools::get_all_before_strict;

use crate::*;

pub struct Translatable {
    original: String,
    origin: (String, usize),
    context: String,
}

impl Element {
    pub(crate) fn get_translatables(&self) -> Vec<Translatable> {
        let mut translatables = Vec::new();
        for child in &self.children {
            match child {
                HtmlPart::Text(text) => translatables.push(Translatable {
                    original: text.to_string(),
                    origin: (String::from("src/unknown.rs"), 0),
                    context: String::from("context unknown"),
                }),
                HtmlPart::Element(el) => translatables.append(&mut el.get_translatables()),
            }
        }
        translatables
    }
}

impl Translatable {
    fn generate_pot_part(&self) -> String {
        format!("#: {}:{}\nmsgctxt {:?}\nmsgid {:?}\nmsgstr \"\"", self.origin.0, self.origin.1, self.context, self.original)
    }
}

pub(crate) fn generate_pot(root: &Element) {
    let translatables = root.get_translatables();
    let pot = translatables.iter().map(|t| t.generate_pot_part()).collect::<Vec<_>>().join("\n\n");
    std::fs::write("test.pot", pot).unwrap();
}

#[derive(Debug)]
pub(crate) enum TextPart {
    Literal(String),
    Variable(String),
}

fn parse_text_part(mut s: &str, args: &Args) -> Vec<TextPart> {
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
        parts.push(TextPart::Variable(var.to_string()));
    }
    if !s.is_empty() {
        parts.push(TextPart::Literal(s.to_string()));
    }

    parts
}

#[derive(Debug)]
pub(crate) struct Catalog {
    catalogs: HashMap<String, HashMap<(String, String), String>>,
    default_language: String,
}

impl Catalog {
    pub(crate) fn new(languages: &[&str], default_language: &str) -> Self {
        let mut catalogs = HashMap::new();
        for language in languages {
            let file = std::fs::File::open(format!("locales/{language}.po")).unwrap_or_else(|_| panic!("could not open the {language} catalog"));
            let parser = PoParser::new();
            let reader = parser.parse(file).unwrap_or_else(|_| panic!("could not parse the {language} catalog"));

            let mut items = HashMap::new();
            for unit in reader {
                let Ok(unit) = unit else {
                    eprintln!("WARNING: Invalid unit in the {language} catalog");
                    continue;
                };

                let context = unit.context().unwrap_or("").to_string();
                if let Message::Simple { id, text: Some(text) } = unit.message() {
                    items.insert((context, id.to_owned()), text.to_owned());
                }
            }
        
            catalogs.insert(language.to_string(), items);
        }
    
        Self {
            catalogs,
            default_language: default_language.to_string(),
        }
    }

    pub(crate) fn translate_text(&self, text: &str, args: &Args) -> Vec<(String, Vec<TextPart>)> {
        let context = String::from("context unknown");
        let context_and_text = (context.clone(), text.to_string());

        let mut translations = Vec::new();
        translations.push((self.default_language.clone(), parse_text_part(text, args)));
        for (language, catalog) in &self.catalogs {
            let Some(translated_text) = catalog.get(&context_and_text) else {
                eprintln!("WARNING: Missing translation for text {text:?} with context {context:?} in language {language}");
                continue;
            };
            let translated_parts = parse_text_part(translated_text, args);
            translations.push((language.to_owned(), translated_parts));
        }

        translations
    }
}
