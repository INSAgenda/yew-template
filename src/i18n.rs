use std::{collections::HashMap, path::Path, io::Read};
use poreader::{PoParser, Message};
use crate::*;

pub struct Translatable {
    original: String,
    origin: (String, usize),
    context: String,
}

impl Element {
    pub(crate) fn get_translatables(&self, args: &Args) -> Vec<Translatable> {
        let mut translatables = Vec::new();
        for child in &self.children {
            match &child.part {
                HtmlPart::Text(text) => translatables.push(Translatable {
                    original: text.to_string(),
                    origin: (args.path.trim_start_matches("./").to_owned(), child.line),
                    context: format!("in {}", args.path.split('/').last().unwrap_or_default().trim_end_matches(".html")),
                }),
                HtmlPart::Element(el) => translatables.append(&mut el.get_translatables(args)),
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

pub(crate) fn generate_pot(root: &Element, args: &Args) {
    let path = Path::new(&args.config.locale_directory);
    if !path.exists() {
        return;
    }

    // Delete template.pot if it hasn't been modified too recently (otherwise keep the data)
    let mut data = match std::fs::File::open(format!("{}template.pot", args.config.locale_directory)) {
        Ok(mut file) => {
            let metadata = file.metadata().unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            if metadata.modified().unwrap().elapsed().unwrap().as_secs() > 120 {
                std::fs::remove_file(format!("{}template.pot", args.config.locale_directory)).unwrap();
                String::new()
            } else {
                data
            }
        }
        _ => String::new()
    };

    // Append new translatables
    let translatables = root.get_translatables(args);
    for translatable in translatables {
        let pot_part = translatable.generate_pot_part();
        if !data.contains(&pot_part) {
            data.push('\n');
            data.push_str(&pot_part);
            data.push('\n');
        }
    }
    std::fs::write(format!("{}template.pot", args.config.locale_directory), data).unwrap();

    // Make sure the file is in .gitignore
    let gitignore_path = format!("{}.gitignore", args.config.locale_directory);
    let path = Path::new(&gitignore_path);
    if !path.exists() {
        std::fs::write(gitignore_path, "template.pot\n").unwrap();
    }
}

#[derive(Debug)]
pub(crate) struct Catalog {
    catalogs: HashMap<String, HashMap<(String, String), String>>,
}

impl Catalog {
    pub(crate) fn new(locale_directory: &str) -> Self {
        // Read all PO files in the locale_directory
        let mut catalogs = HashMap::new();
        let read_dir = match std::fs::read_dir(locale_directory) {
            Ok(read_dir) => read_dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self { catalogs },
            Err(_) => abort_call_site!("Failed to read locale directory"),
        };
        for entry in read_dir {
            let entry = entry.expect("Error while reading locale directory");
            let path = entry.path();
            if path.extension().map(|ext| ext != "po").unwrap_or(true) {
                continue;
            }

            let locale = path.file_name().expect("no file stem").to_str().expect("cannot convert file stem").trim_end_matches(".po").to_string();
            let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("could not open the {locale} catalog"));
            let parser = PoParser::new();
            let reader = parser.parse(file).unwrap_or_else(|_| panic!("could not parse the {locale} catalog"));

            let mut items = HashMap::new();
            for unit in reader {
                let Ok(unit) = unit else {
                    eprintln!("WARNING: Invalid unit in the {locale} catalog");
                    continue;
                };
                let context = unit.context().unwrap_or("").to_string();
                if let Message::Simple { id, text: Some(text) } = unit.message() {
                    items.insert((context, id.to_owned()), text.to_owned());
                }
            }
        
            catalogs.insert(locale.to_string(), items);
        }
    
        Self {
            catalogs,
        }
    }

    pub(crate) fn translate_text(&self, text: &str, args: &Args) -> Vec<(String, Vec<TextPart>)> {
        let context = String::from("context unknown");
        let context_and_text = (context.clone(), text.to_string());

        let mut translations = Vec::new();
        translations.push((String::new(), parse_text_part(text, args)));
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
