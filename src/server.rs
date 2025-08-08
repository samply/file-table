#![cfg(feature = "server")]

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub fhir_base_url: String,
    pub fhir_username: Option<String>,
    pub fhir_password: Option<String>,
}

static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// Load the configuration from scout.toml. Should be called once on server startup.
pub fn load_config() -> anyhow::Result<()> {
    let config_str = std::fs::read_to_string("scout.toml")?;
    let config = toml::from_str(&config_str)?;
    CONFIG.set(config).expect("Config should only be set once");
    Ok(())
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config should be loaded before use")
}

impl crate::serverfn::RequestBuilderExt for reqwest::RequestBuilder {
    fn with_auth(self) -> Self {
        if let Some(fhir_username) = &config().fhir_username {
            self.basic_auth(fhir_username, config().fhir_password.as_deref())
        } else {
            self
        }
    }
}
