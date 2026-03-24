use std::path::Path;

use dioxus::prelude::*;

use crate::{
    app_state::{AppState, MobileScreen},
    mobile_editor::{render_editor, render_tilesets},
    mobile_pages::{render_dashboard, render_layers, render_objects, render_settings},
    session_ops::load_sample,
};

pub(crate) fn render_mobile_shell(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-shell",
            match snapshot.mobile_screen {
                MobileScreen::Dashboard => rsx! { {render_dashboard(snapshot, state)} },
                MobileScreen::Editor => rsx! { {render_editor(snapshot, state)} },
                MobileScreen::Tilesets => rsx! { {render_tilesets(snapshot, state)} },
                MobileScreen::Layers => rsx! { {render_layers(snapshot, state)} },
                MobileScreen::Objects => rsx! { {render_objects(snapshot, state)} },
                MobileScreen::Settings => rsx! { {render_settings(snapshot, state)} },
            }
        }
    }
}

pub(crate) fn render_detail_header(title: &'static str, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-page-header",
            button {
                class: "mobile-inline-action",
                onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                "Back"
            }
            h1 { "{title}" }
            button {
                class: "mobile-inline-action",
                onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                "Done"
            }
        }
    }
}

pub(crate) fn render_bottom_nav(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-bottom-nav",
            {bottom_nav_button(snapshot, state, MobileScreen::Dashboard, "Projects")}
            {bottom_nav_button(snapshot, state, MobileScreen::Tilesets, "Tilesets")}
            {bottom_nav_button(snapshot, state, MobileScreen::Layers, "Layers")}
            {bottom_nav_button(snapshot, state, MobileScreen::Objects, "Objects")}
            {bottom_nav_button(snapshot, state, MobileScreen::Settings, "Settings")}
        }
    }
}

pub(crate) fn render_missing_state(
    mut state: Signal<AppState>,
    message: &'static str,
) -> Element {
    rsx! {
        div { class: "mobile-screen",
            div { class: "mobile-page-header",
                button {
                    class: "mobile-inline-action",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Dashboard,
                    "Back"
                }
                h1 { "Unavailable" }
                span { class: "mobile-inline-action ghost", "Soon" }
            }
            div { class: "mobile-page-body",
                div { class: "mobile-placeholder-card",
                    strong { "No map loaded" }
                    p { "{message}" }
                    button {
                        onclick: move |_| {
                            let mut state = state.write();
                            load_sample(&mut state);
                            state.mobile_screen = MobileScreen::Editor;
                        },
                        "Load Demo"
                    }
                }
            }
        }
    }
}

pub(crate) fn document_title(snapshot: &AppState) -> String {
    Path::new(&snapshot.path_input)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Embedded Demo")
        .to_string()
}

pub(crate) fn layer_kind_label(layer: &wishing_core::Layer) -> &'static str {
    if layer.as_tile().is_some() {
        "Tile Layer"
    } else {
        "Object Layer"
    }
}

fn bottom_nav_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    screen: MobileScreen,
    label: &'static str,
) -> Element {
    let class = if snapshot.mobile_screen == screen {
        "active"
    } else {
        ""
    };
    rsx! {
        button {
            class: class,
            onclick: move |_| state.write().mobile_screen = screen.clone(),
            span { class: "mobile-nav-pill" }
            span { "{label}" }
        }
    }
}
