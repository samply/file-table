use dioxus::prelude::*;
use fhir::TimelineEvent;
use itertools::Itertools;

mod fhir;
mod server;
mod serverfn;
mod table;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    PatientTable {},
    #[route("/patient/:id")]
    PatientView { id: String },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::initialize_default();

    #[cfg(feature = "server")]
    if let Err(e) = server::load_config() {
        tracing::error!("Failed to load config: {e}");
        std::process::exit(1);
    }

    #[cfg(feature = "server")]
    if let Err(e) = server::load_code_maps() {
        tracing::error!("Failed to load code maps: {e}");
        std::process::exit(1);
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load polyfill for CSS anchor positioning if needed (https://github.com/oddbird/css-anchor-positioning)
    document::eval(
        "if (!('anchorName' in document.documentElement.style)) import('https://unpkg.com/@oddbird/css-anchor-positioning/dist/css-anchor-positioning-fn.js').then(mod => {window.CSSAnchorPositioning = mod.default; window.CSSAnchorPositioning()});",
    );

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn PatientTable() -> Element {
    let patients = use_server_future(|| serverfn::get_patients())?;
    match &*patients.read_unchecked() {
        Some(Ok(patients)) => rsx! {
            table::Table {
                columns: vec![
                    table::Column::new("ID").hidden(),
                    table::Column::new("Gender").categorical(),
                    table::Column::new("Birth Date"),
                    table::Column::new("Deceased").categorical(),
                    table::Column::new("Address"),
                ],
                data: patients
                    .iter()
                    .map(|p| vec![p.id(), p.gender(), p.birth_date(), p.deceased(), p.address()])
                    .collect(),
                ondetail: {
                    let patients = patients.clone();
                    move |id: usize| {
                        let id = patients[id].id();
                        navigator().push(Route::PatientView { id });
                    }
                },
            }
        },
        Some(Err(e)) => rsx! { "Error loading patients: {e:#}" },
        None => rsx! { "Loading..." },
    }
}

#[component]
fn OptionalChip(chip: Option<fhir::Chip>) -> Element {
    rsx! {
        if let Some(chip) = chip {
            span {
                class: "text-sm border rounded-full px-1.5 {chip.class}",
                title: "{chip.hover_text}",
                "{chip.text}"
            }
        }
    }
}

#[component]
fn CodeableConcept(codeable_concept: fhir::CodeableConcept) -> Element {
    // Put user-selected coding first
    let codings = codeable_concept
        .coding
        .iter()
        .flatten()
        .sorted_by_key(|c| c.user_selected)
        .rev()
        .collect::<Vec<_>>();

    let text = codeable_concept
        .text
        .or_else(|| {
            codings
                .first()
                .and_then(|c| c.display.clone().or(c.code.clone()))
        })
        .unwrap_or_default();

    rsx! {
        "{text}"
        for coding in codings {
            " "
            span {
                class: "text-sm border rounded-full px-1.5 bg-blue-100 border-blue-500",
                title: "{coding.system.clone().unwrap_or_default()}",
                "{coding.code.clone().unwrap_or_default()}"
            }
        }
    }
}

#[component]
fn PatientView(id: String) -> Element {
    let id = use_signal(|| id);
    let patient_details = use_server_future(move || serverfn::get_patient_details(id()))?;
    match &*patient_details.read_unchecked() {
        Some(Ok((patient, bundle))) => rsx! {
            div { class: "max-w-4xl mx-auto px-4",
                h2 { class: "text-xl font-bold my-3", "Patient Details" }
                p { "Gender: {patient.gender()}" }
                p { "Birth Date: {patient.birth_date()}" }
                p { "Deceased: {patient.deceased()}" }
                p { "Address: {patient.address()}" }
                h2 { class: "text-xl font-bold my-3", "Patient Timeline" }
                ol { class: "relative border-s border-gray-300",
                    for entry in bundle
                        .entry
                        .iter()
                        .filter(|e| e.resource.timeline_event().is_some())
                        .sorted_by_key(|e| e.resource.timeline_event().unwrap().timestamp())
                    {
                        li { class: "mb-5 ms-4",
                            div { class: "absolute w-3 h-3 bg-gray-300 rounded-full mt-1.5 -start-1.5 border border-white" }
                            match entry.resource {
                                fhir::Resource::Encounter(ref encounter) => {
                                    rsx! {
                                        div { class: "flex items-center gap-1.5",
                                            h3 { class: "font-bold", "Encounter" }
                                            OptionalChip { chip: encounter.status_chip() }
                                        }
                                        time { class: "text-sm text-gray-600", "{encounter.formatted_timestamp()}" }
                                        p { "Class: {encounter.class()}" }
                                        p { "Visit number: {encounter.visit_number()}" }
                                        p { "Encounter level: {encounter.encounter_level()}" }
                                        p { "Service type: {encounter.service_type()}" }
                                        p { "Service provider: {encounter.service_provider()}" }
                                    }
                                }
                                fhir::Resource::Condition(ref condition) => {
                                    rsx! {
                                        div { class: "flex items-center gap-1.5",
                                            h3 { class: "font-bold", "Condition" }
                                            OptionalChip { chip: condition.clinical_status_chip() }
                                            OptionalChip { chip: condition.verification_status_chip() }
                                        }
                                        time { class: "text-sm text-gray-600", "{condition.formatted_timestamp()}" }
                                        p {
                                            "Code: "
                                            CodeableConcept { codeable_concept: condition.code.clone() }
                                        }
                                    }
                                }
                                fhir::Resource::Procedure(ref procedure) => {
                                    rsx! {
                                        div { class: "flex items-center gap-1.5",
                                            h3 { class: "font-bold", "Procedure" }
                                            OptionalChip { chip: procedure.status_chip() }
                                        }
                                        time { class: "text-sm text-gray-600", "{procedure.formatted_timestamp()}" }
                                        p { "Category: {procedure.category()}" }
                                        p {
                                            "Code: "
                                            CodeableConcept { codeable_concept: procedure.code.clone() }
                                        }
                                    }
                                }
                                fhir::Resource::Observation(ref observation) => {
                                    rsx! {
                                        div { class: "flex items-center gap-1.5",
                                            h3 { class: "font-bold", "Lab result" }
                                            OptionalChip { chip: observation.status_chip() }
                                        }
                                        time { class: "text-sm text-gray-600", "{observation.formatted_timestamp()}" }
                                        p {
                                            "Code: "
                                            CodeableConcept { codeable_concept: observation.code.clone() }
                                        }
                                        p { "ID: {observation.identifier()}" }
                                        p {
                                            "Value: {observation.value()} "
                                            OptionalChip { chip: observation.interpretation_chip() }
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        },
        Some(Err(e)) => rsx! { "Error loading patient: {e:#}" },
        None => rsx! { "Loading..." },
    }
}
