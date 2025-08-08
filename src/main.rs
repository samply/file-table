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
fn PatientView(id: String) -> Element {
    let id = use_signal(|| id);
    let patient_details = use_server_future(move || serverfn::get_patient_details(id()))?;
    match &*patient_details.read_unchecked() {
        Some(Ok((patient, bundle))) => rsx! {
            div { class: "m-4",
                h2 { class: "text-xl font-bold my-3", "Patient Details" }
                // p { "Name: {patient.name()}" }
                p { "Gender: {patient.gender()}" }
                p { "Birth Date: {patient.birth_date()}" }
                p { "Deceased: {patient.deceased()}" }
                p { "Address: {patient.address()}" }
                h2 { class: "text-xl font-bold my-3", "Patient Timeline" }
                // p {
                //     class: "flex gap-1.5",
                //     svg {
                //         stroke: "currentColor",
                //         fill: "none",
                //         xmlns: "http://www.w3.org/2000/svg",
                //         "stroke-width": "1.5",
                //         "viewBox": "0 0 24 24",
                //         class: "size-6",
                //         path {
                //             "stroke-linejoin": "round",
                //             "stroke-linecap": "round",
                //             d: "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z",
                //         }
                //     }
                //     "4 events are not shown because they are missing a timestamp."
                // }
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
                                        details { open: false,
                                            summary {
                                                div { class: "inline-flex items-center gap-1.5",
                                                    h3 { class: "font-bold", "Encounter" }
                                                    OptionalChip { chip: encounter.status_chip() }
                                                }
                                            }
                                            time { class: "my-0.5 text-sm font-normal leading-none text-gray-600",
                                                "{encounter.formatted_timestamp()}"
                                            }
                                            p { "Class: {encounter.class()}" }
                                            p { "Visit number: {encounter.visit_number()}" }
                                            p { "Encounter level: {encounter.encounter_level()}" }
                                            p { "Service type: {encounter.service_type()}" }
                                            p { "Service provider: {encounter.service_provider()}" }
                                        }
                                    }
                                }
                                fhir::Resource::Condition(ref condition) => {
                                    rsx! {
                                        details { open: true,
                                            summary {
                                                div { class: "inline-flex items-center gap-1.5",
                                                    h3 { class: "font-bold", "Condition" }
                                                    OptionalChip { chip: condition.clinical_status_chip() }
                                                    OptionalChip { chip: condition.verification_status_chip() }
                                                }
                                            }
                                            time { class: "my-0.5 text-sm font-normal leading-none text-gray-600",
                                                "{condition.formatted_timestamp()}"
                                            }
                                            p { "Code: {condition.code()}" }
                                            p { "Body site: {condition.body_site()}" }
                                            p { "Onset: {condition.onset_start()}" }
                                        // p { "Notes: {condition.notes()}" }
                                        }
                                    }
                                }
                                fhir::Resource::Procedure(ref procedure) => {
                                    rsx! {
                                        details { open: true,
                                            summary {
                                                div { class: "inline-flex items-center gap-1.5",
                                                    h3 { class: "font-bold", "Procedure" }
                                                    OptionalChip { chip: procedure.status_chip() }
                                                }
                                            }
                                            time { class: "my-0.5 text-sm font-normal leading-none text-gray-600",
                                                "{procedure.formatted_timestamp()}"
                                            }
                                            p { "Category: {procedure.category()}" }
                                            p { "Code: {procedure.code()}" }
                                            p { "Body Site: {procedure.body_site()}" }
                                        // p { "Notes: {procedure.note()}" }
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
