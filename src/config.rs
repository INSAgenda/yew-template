use crate::*;

#[cfg_attr(feature = "config", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// When arguments required by the template are missing, attempt to use the local variable with the same name instead of aborting.
    pub auto_default: bool,

    /// Path to the directory containing the templates.
    /// Expected to be relative to the crate root.
    pub template_folder: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_default: true,
            template_folder: String::new(),
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
    if !config.template_folder.is_empty() && !config.template_folder.ends_with('/') {
        config.template_folder.push('/');
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
