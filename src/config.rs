use crate::*;

#[cfg_attr(feature = "config", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "config", serde(untagged))]
pub enum AnyValues {
    Value(String),
    Values(Vec<String>),
}

impl AnyValues {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            AnyValues::Value(v) => vec![v],
            AnyValues::Values(v) => v,
        }
    }
}

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

    /// Helpers to use in templates
    #[cfg_attr(feature = "config", serde(skip_serializing))]
    pub helpers: HashMap<String, HashMap<usize, Helper>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut helpers = HashMap::new();
        helpers.insert(
            String::from("loud"),
            vec![Helper::parse("[0].to_uppercase()")]
                .into_iter()
                .collect(),
        );
        helpers.insert(
            String::from("message"),
            vec![Helper::parse("ctx.link().callback(|_| [0])")]
                .into_iter()
                .collect(),
        );

        Self {
            auto_default: false,
            template_directory: String::from("./"),
            locale_directory: String::from("./locales/"),
            locale_code: String::from("locale.as_str()"),
            variable_bounds: (String::from("{{"), String::from("}}")),
            helpers,
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
    pub helpers: Option<HashMap<String, AnyValues>>,
}

impl From<ConfigLoader> for Config {
    fn from(val: ConfigLoader) -> Self {
        let default = Config::default();
        let mut helpers = default.helpers;
        if let Some(custom_helpers) = val.helpers {
            for helper_name in custom_helpers {
                for helper_def in helper_name.1.into_vec() {
                    let (args_len, helper) = Helper::parse(&helper_def);
                    helpers
                        .entry(helper_name.0.clone())
                        .or_insert_with(HashMap::new)
                        .insert(args_len, helper);
                }
            }
        }

        Config {
            auto_default: val.auto_default.unwrap_or(default.auto_default),
            template_directory: val.template_directory.unwrap_or(default.template_directory),
            locale_directory: val.locale_directory.unwrap_or(default.locale_directory),
            locale_code: val.locale_code.unwrap_or(default.locale_code),
            variable_bounds: val.variable_separator.unwrap_or(default.variable_bounds),
            helpers,
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
