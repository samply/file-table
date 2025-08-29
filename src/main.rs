use dioxus::prelude::*;

mod server;
mod serverfn;
mod table;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    FileTable {}
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::initialize_default();

    // #[cfg(feature = "server")]
    // if let Err(e) = server::load_config() {
    //     tracing::error!("Failed to load config: {e}");
    //     std::process::exit(1);
    // }

    dioxus::launch(App);
}

pub fn format_timestamp(timestamp: jiff::Timestamp) -> String {
    let zoned = timestamp.to_zoned(jiff::tz::TimeZone::system());
    // Jan 08, 2020, 07:00 CET
    zoned.strftime("%b %d, %Y, %H:%M %Z").to_string()
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
fn FileTable() -> Element {
    let files = use_server_future(|| serverfn::get_files())?;
    match &*files.read_unchecked() {
        Some(Ok(files)) => rsx! {
            table::Table {
                columns: vec![table::Column::new("Name")],
                data: files.iter().map(|f| vec![f.name.clone()]).collect(),
                ondetail: {
                    let files = files.clone();
                    move |id: usize| {
                        tracing::info!("User clicked detail for file: {}", files[id].name)
                    }
                },
            }
        },
        Some(Err(e)) => rsx! { "Error loading files: {e:#}" },
        None => rsx! { "Loading..." },
    }
}
