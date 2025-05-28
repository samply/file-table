use dioxus::prelude::*;

use crate::fhir;

#[server]
pub async fn get_patients() -> Result<Vec<fhir::Patient>, ServerFnError> {
    let url = format!(
        "{}/Patient",
        std::env::var("FHIR_BASE_URL").unwrap_or("http://127.0.0.1:8081/fhir".into())
    );
    let client = reqwest::Client::new();
    let bundle = client
        .get(&url)
        .basic_auth(
            std::env::var("FHIR_USERNAME").unwrap_or_default(),
            Some(std::env::var("FHIR_PASSWORD").unwrap_or_default()),
        )
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

pub async fn get_resources<T>(resource_type: &str) -> Result<Vec<T>, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    let url = format!(
        "{}/{}",
        std::env::var("FHIR_BASE_URL").unwrap_or("http://127.0.0.1:8081/fhir".into()),
        resource_type
    );
    let client = reqwest::Client::new();
    let bundle = client
        .get(&url)
        .basic_auth(
            std::env::var("FHIR_USERNAME").unwrap_or_default(),
            Some(std::env::var("FHIR_PASSWORD").unwrap_or_default()),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<fhir::FhirBundle<T>>()
        .await?;
    Ok(bundle
        .entry
        .into_iter()
        .map(|entry| entry.resource)
        .collect())
}

pub async fn get_resource<T>(resource_type: &str, id: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let url = format!(
        "{}/{}/{}",
        std::env::var("FHIR_BASE_URL").unwrap_or("http://127.0.0.1:8081/fhir".into()),
        resource_type,
        id
    );
    let client = reqwest::Client::new();
    let resource = client
        .get(&url)
        .basic_auth(
            std::env::var("FHIR_USERNAME").unwrap_or_default(),
            Some(std::env::var("FHIR_PASSWORD").unwrap_or_default()),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<T>()
        .await?;
    Ok(resource)
}

/// Get a patient and their related resources. For now returns the patient and their encounters.
#[server]
pub async fn get_patient_details(
    id: String,
) -> Result<(fhir::Patient, fhir::MixedBundle), ServerFnError> {
    let url = format!(
        "{}/Patient/{}/$everything",
        std::env::var("FHIR_BASE_URL").unwrap_or("http://127.0.0.1:8081/fhir".into()),
        id
    );
    let client = reqwest::Client::new();
    let mut bundle = client
        .get(&url)
        .basic_auth(
            std::env::var("FHIR_USERNAME").unwrap_or_default(),
            Some(std::env::var("FHIR_PASSWORD").unwrap_or_default()),
        )
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
