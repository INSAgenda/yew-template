use crate::*;

#[cfg_attr(feature = "config", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// When arguments required by the template are missing, attempt to use the local variable with the same name instead of aborting.
    pub auto_default: bool,

    /// Path to the directory containing the templates.
    /// Expected to be relative to the crate root.
    pub template_directory: String,

    /// Path to the directory containing PO files.
    /// The name (without extension) of these files will be used to match the locale variable.
    pub locale_directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_default: true,
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
