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
    catalogs: Vec<(String, gettext::Catalog)>,
    default_language: String,
}

impl Catalog {
    pub(crate) fn new(languages: Vec<String>, default_language: String) -> Self {
        let mut catalogs = Vec::new();
        for language in languages {
            let f = std::fs::File::open("locales/fr.mo").expect("could not open the catalog");
            let catalog = gettext::Catalog::parse(f).expect("could not parse the catalog");
            println!("{:#?}", catalog);
            catalogs.push((language, catalog));
        }
    
        Self {
            catalogs,
            default_language
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.catalogs.is_empty()
    }

    pub(crate) fn translate_text(&self, text: &str, args: &Args) -> Vec<(String, Vec<TextPart>)> {
        let context = String::from("context unknown");
        let id = format!("{context}\u{4}{text}");

        let mut translations = Vec::new();
        translations.push((self.default_language.clone(), parse_text_part(text, args)));
        for (language, catalog) in &self.catalogs {
            let mut translated_text = catalog.gettext(&id);
            if translated_text == id {
                translated_text = text;
            }
            let translated_parts = parse_text_part(translated_text, args);
            translations.push((language.to_owned(), translated_parts));
        }

        translations
    }
}
