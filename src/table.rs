use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TableProps {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub ondetail: EventHandler<String>,
}

#[component]
pub fn Table(props: TableProps) -> Element {
    let mut search_text = use_signal(|| "".to_string());
    let filtered_rows = use_memo(move || {
        let search_text = search_text.read().to_lowercase();
        props
            .rows
            .iter()
            .filter(|row| {
                row.iter()
                    .any(|cell| cell.to_lowercase().contains(&search_text))
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    rsx! {
        div {
            class: "m-4",
            input {
                class: "border border-gray-300 rounded p-1",
                placeholder: "Search table...",
                value: "{search_text}",
                oninput: move |event| search_text.set(event.value()),
            }
        }
        div {
            class: "grid gap-px p-px m-4",
            style: "grid-template-columns: repeat({props.headers.len()}, auto)",
            div {
                class: "grid grid-cols-subgrid col-span-full",
                for header in props.headers.iter() {
                    div {
                        class: "outline outline-gray-300 p-2 bg-gray-100 font-bold",
                        "{header}"
                    }
                }
            }
            for row in filtered_rows().into_iter() {
                div {
                    class: "grid grid-cols-subgrid col-span-full",
                    for cell in row.iter() {
                        div {
                            class: "outline outline-gray-300 p-2",
                            "{cell}"
                        }
                    }
                    button {
                        class: "outline outline-gray-300 p-2",
                        onclick: move |_| {
                            (props.ondetail)(row[0].clone());
                        },
                        svg {
                            "viewBox": "0 0 24 24",
                            "stroke-width": "1.5",
                            xmlns: "http://www.w3.org/2000/svg",
                            stroke: "currentColor",
                            fill: "none",
                            class: "size-6",
                            path {
                                "stroke-linejoin": "round",
                                d: "m8.25 4.5 7.5 7.5-7.5 7.5",
                                "stroke-linecap": "round",
                            }
                        }
                    }
                }
            }
        }
    }
}
