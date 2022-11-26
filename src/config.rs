use crate::*;

#[derive(Debug)]
#[cfg_attr(feature = "config", derive(serde::Serialize))]
pub struct Config {
    /// Whether to attempt to capture local variables instead of aborting when arguments required by the template are missing.
    pub auto_default: bool,

    /// Where to look for templates (relative to crate root)
    pub template_directory: String,

    /// Where to look for locales (relative to crate root)
    pub locale_directory: String,

    /// Rust code to evaluate as locale. Should evaluate to a &str.
    /// If will be inserted in generated code like this: `match locale_code {`.
    pub locale_code: String,

    /// Two strings marking the beginning and end of a variable in a template.
    pub variable_bounds: (String, String),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_default: false,
            template_directory: String::from("./"),
            locale_directory: String::from("./locales/"),
            locale_code: String::from("locale.as_str()"),
            variable_bounds: (String::from("{{"), String::from("}}")),
        }
    }
}

#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct ConfigLoader {
    pub auto_default: Option<bool>,
    pub template_directory: Option<String>,
    pub locale_directory: Option<String>,
    pub locale_code: Option<String>,
    pub variable_separator: Option<(String, String)>,
}

impl From<ConfigLoader> for Config {
    fn from(val: ConfigLoader) -> Self {
        let default = Config::default();
        Config {
            auto_default: val.auto_default.unwrap_or(default.auto_default),
            template_directory: val.template_directory.unwrap_or(default.template_directory),
            locale_directory: val.locale_directory.unwrap_or(default.locale_directory),
            locale_code: val.locale_code.unwrap_or(default.locale_code),
            variable_bounds: val.variable_separator.unwrap_or(default.variable_bounds),
        }
    }
}

#[cfg(feature = "config")]
pub fn read_config() -> Config {
    let Ok(data) = std::fs::read_to_string("yew-template.toml") else {
        return Config::default();
    };

    let Ok(config_loader) = toml::from_str::<ConfigLoader>(&data) else {
        abort_call_site!("Failed to parse yew-template.toml");
    };
    let mut config: Config = config_loader.into();

    if !config.template_directory.is_empty() && !config.template_directory.ends_with('/') {
        config.template_directory.push('/');
    }
    if !config.locale_directory.is_empty() && !config.locale_directory.ends_with('/') {
        config.locale_directory.push('/');
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

#[cfg(feature = "config")]
#[test]
fn print_default_config() {
    println!("{}", toml::to_string_pretty(&Config::default()).unwrap());
}
