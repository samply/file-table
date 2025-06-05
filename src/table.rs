use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TableProps {
    pub columns: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub ondetail: EventHandler<String>,
}

#[component]
pub fn Table(props: TableProps) -> Element {
    let columns = use_signal(|| props.columns.clone());
    let mut search_text = use_signal(|| "".to_string());
    let mut custom_columns = use_signal(|| props.columns.clone());
    let filtered_data = use_memo(move || {
        let search_text = search_text.read().to_lowercase();
        props
            .data
            .iter()
            .filter(|row| {
                row.iter()
                    .any(|cell| cell.to_lowercase().contains(&search_text))
            })
            .map(|row| {
                custom_columns
                    .read()
                    .iter()
                    .filter_map(|header| {
                        columns
                            .read()
                            .iter()
                            .position(|h| h == header)
                            .and_then(|idx| row.get(idx).cloned())
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    rsx! {
        div {
            class: "m-4 flex items-center gap-2",
            input {
                class: "border border-gray-300 rounded p-1",
                placeholder: "Search table...",
                value: "{search_text}",
                oninput: move |event| search_text.set(event.value()),
            }
            button {
                popovertarget: "customize",
                class: "border border-gray-300 rounded p-1 bg-gray-100 hover:bg-gray-200 [anchor-name:--customize]",
                "Customize Columns"
            }
            div {
                id: "customize",
                popover: "auto",
                // inset-auto is only required for the anchor positioning polyfill
                class: "border border-gray-300 rounded shadow-md p-2 absolute [position-anchor:--customize] [position-area:bottom_center] inset-auto",
                for header in props.columns.iter().cloned() {
                    label {
                        class: "flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: custom_columns().contains(&header),
                            onchange: move |_| {
                                custom_columns.with_mut(|vec| {
                                    if vec.contains(&header) {
                                        vec.retain(|h| h != &header);
                                    } else {
                                        // Insert header in the correct order as in props.columns
                                        let pos = columns.read().iter().position(|h| h == &header).unwrap();
                                        let mut insert_at = vec.len();
                                        for (i, h) in vec.iter().enumerate() {
                                            if let Some(col_pos) = columns.read().iter().position(|c| c == h) {
                                                if col_pos > pos {
                                                    insert_at = i;
                                                    break;
                                                }
                                            }
                                        }
                                        vec.insert(insert_at, header.clone());
                                    }
                                });
                            },
                        }
                        span { "{header}" }
                    }
                }
            }
        }
        div {
            class: "grid gap-px p-px m-4",
            style: "grid-template-columns: repeat({custom_columns().len() + 1}, auto)",
            div {
                class: "grid grid-cols-subgrid col-span-full",
                for header in custom_columns().iter() {
                    div {
                        class: "outline outline-gray-300 p-2 bg-gray-100 font-bold",
                        "{header}"
                    }
                }
                div {
                    class: "outline outline-gray-300 p-2 bg-gray-100"
                }
            }
            for row in filtered_data().into_iter() {
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
