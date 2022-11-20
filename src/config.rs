use crate::*;

#[derive(Debug)]
#[cfg_attr(feature = "config", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// Whether to attempt to capture local variables instead of aborting when arguments required by the template are missing.
    pub auto_default: bool,

    /// Where to look for templates (relative to crate root)
    pub template_directory: String,

    /// Where to look for locales (relative to crate root)
    pub locale_directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_default: false,
            template_directory: String::from("./"),
            locale_directory: String::from("./locales/"),
        }
    }
}

#[cfg(feature = "config")]
pub fn read_config() -> Config {
    let Ok(data) = std::fs::read_to_string("yew-template.toml") else {
        return Config::default();
    };

    let Ok(mut config) = toml::from_str::<Config>(&data) else {
        abort_call_site!("Failed to parse yew-template.toml");
    };
    if !config.template_directory.is_empty() && !config.template_directory.ends_with('/') {
        config.template_directory.push('/');
    }

    config
}

#[cfg(not(feature = "config"))]
pub fn read_config() -> Config {
    if std::fs::File::open("yew-template.toml").is_ok() {
        abort_call_site!("yew-template.toml found but the \"config\" feature is not enabled");
    }
    Config::default()
}

#[test]
fn print_default_config() {
    println!("{}", toml::to_string_pretty(&Config::default()).unwrap());
}
