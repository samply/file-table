use dioxus::prelude::*;

use crate::fhir;

#[cfg(feature = "server")]
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub fhir_base_url: String,
    pub fhir_username: Option<String>,
    pub fhir_password: Option<String>,
}

#[cfg(feature = "server")]
static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// Load the configuration from scout.toml. Should be called once on server startup.
#[cfg(feature = "server")]
pub fn load_config() -> anyhow::Result<()> {
    let config_str = std::fs::read_to_string("scout.toml")?;
    let config = toml::from_str(&config_str)?;
    CONFIG.set(config).expect("Config should only be set once");
    Ok(())
}

#[cfg(feature = "server")]
pub fn config() -> &'static Config {
    CONFIG.get().expect("Config should be loaded before use")
}

#[cfg(feature = "server")]
pub trait RequestBuilderExt {
    fn with_auth(self) -> Self;
}

#[cfg(feature = "server")]
impl RequestBuilderExt for reqwest::RequestBuilder {
    fn with_auth(self) -> Self {
        if let Some(fhir_username) = &config().fhir_username {
            self.basic_auth(fhir_username, config().fhir_password.as_deref())
        } else {
            self
        }
    }
}

#[server]
pub async fn get_patients() -> Result<Vec<fhir::Patient>, ServerFnError> {
    let url = format!("{}/Patient", config().fhir_base_url);
    let client = reqwest::Client::new();
    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<fhir::FhirBundle<fhir::Patient>>()
        .await?;
    Ok(bundle
        .entry
        .into_iter()
        .map(|entry| entry.resource)
        .collect())
}

/// Get a patient and their related resources.
#[server]
pub async fn get_patient_details(
    id: String,
) -> Result<(fhir::Patient, fhir::MixedBundle), ServerFnError> {
    let url = format!("{}/Patient/{}/$everything", config().fhir_base_url, id);
    let client = reqwest::Client::new();

    let bundle = client
        .get(&url)
        .with_auth()
        .send()
        .await?
        .error_for_status()?
        .json::<fhir::MixedBundle>()
        .await?;

    let patient = bundle
        .entry
        .iter()
        .find_map(|entry| {
            if let fhir::Resource::Patient(patient) = &entry.resource {
                Some(patient.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| ServerFnError::new("No patient found"))?;

    Ok((patient, bundle))
}
