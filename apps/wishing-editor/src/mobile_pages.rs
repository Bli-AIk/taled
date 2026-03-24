use dioxus::prelude::*;
use wishing_core::{EditorSession, ObjectShape};

use crate::{
    app_state::{AppState, MobileScreen},
    edit_ops::{
        create_object, delete_selected_object, nudge_selected_object, rename_selected_object,
        selected_object_view, toggle_layer_lock, toggle_layer_visibility,
    },
    mobile_shell::{
        document_title, layer_kind_label, render_bottom_nav, render_detail_header,
        render_missing_state,
    },
    session_ops::{adjust_zoom, load_sample, save_document},
    ui_visuals::object_icon_style,
};

#[cfg(target_os = "android")]
use crate::platform::log_path;

#[derive(Clone)]
struct MobileObjectSummary {
    layer_index: usize,
    object_id: u32,
    name: String,
    shape: ObjectShape,
}

pub(crate) fn render_dashboard(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let project_name = document_title(snapshot);
    let has_session = snapshot.session.is_some();

    rsx! {
        div { class: "mobile-screen mobile-dashboard-screen",
            div { class: "mobile-page-header mobile-dashboard-header",
                h1 { "Project Dashboard" }
                button {
                    class: "mobile-inline-action",
                    onclick: move |_| {
                        let mut state = state.write();
                        load_sample(&mut state);
                        state.mobile_screen = MobileScreen::Editor;
                    },
                    "Open Demo"
                }
            }
            div { class: "mobile-page-body",
                button {
                    class: "mobile-dashboard-hero",
                    onclick: move |_| {
                        let mut state = state.write();
                        load_sample(&mut state);
                        state.mobile_screen = MobileScreen::Editor;
                    },
                    div { class: "mobile-dashboard-hero-title", "Open Embedded Demo" }
                    div { class: "mobile-dashboard-hero-meta", "Boot straight into the Android-first editor shell." }
                }

                if has_session {
                    button {
                        class: "mobile-project-card",
                        onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                        div { class: "mobile-project-thumb live" }
                        div { class: "mobile-project-copy",
                            div { class: "mobile-project-title", "{project_name}" }
                            div { class: "mobile-project-meta", "{snapshot.status}" }
                        }
                    }
                }

                div { class: "mobile-section-label", "Planned Workflow" }
                div { class: "mobile-card-list",
                    div { class: "mobile-project-card placeholder",
                        div { class: "mobile-project-thumb placeholder" }
                        div { class: "mobile-project-copy",
                            div { class: "mobile-project-title", "Import TMX" }
                            div { class: "mobile-project-meta", "UI placeholder for a future Android file-picker flow." }
                        }
                    }
                    div { class: "mobile-project-card placeholder",
                        div { class: "mobile-project-thumb placeholder" }
                        div { class: "mobile-project-copy",
                            div { class: "mobile-project-title", "Recent Stitch Concepts" }
                            div { class: "mobile-project-meta", "Reserved for concept boards and design references." }
                        }
                    }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

pub(crate) fn render_layers(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_state(state, "Load the embedded demo before managing layers.");
    };

    rsx! {
        div { class: "mobile-screen",
            {render_detail_header("Layer Manager", state)}
            div { class: "mobile-page-body",
                for (index, layer) in session.document().map.layers.iter().enumerate() {
                    div {
                        key: "mobile-layer-{index}",
                        class: if snapshot.active_layer == index { "mobile-layer-card active" } else { "mobile-layer-card" },
                        div { class: "mobile-layer-card-main",
                            button {
                                class: "mobile-layer-name",
                                onclick: move |_| {
                                    let mut state = state.write();
                                    state.active_layer = index;
                                    state.selected_object = None;
                                },
                                strong { "{layer.name()}" }
                                span { "{layer_kind_label(layer)}" }
                            }
                            div { class: "mobile-layer-actions",
                                button {
                                    class: if layer.visible() { "on" } else { "off" },
                                    onclick: move |_| toggle_layer_visibility(&mut state.write(), index),
                                    if layer.visible() { "Visible" } else { "Hidden" }
                                }
                                button {
                                    class: if layer.locked() { "on" } else { "off" },
                                    onclick: move |_| toggle_layer_lock(&mut state.write(), index),
                                    if layer.locked() { "Locked" } else { "Unlocked" }
                                }
                            }
                        }
                        div { class: "mobile-progress-row",
                            span { "Opacity" }
                            div { class: "mobile-progress-track",
                                div { class: "mobile-progress-fill", style: "width:100%;" }
                            }
                            span { "UI" }
                        }
                    }
                }
                div { class: "mobile-placeholder-card compact",
                    strong { "Add Layer" }
                    p { "UI placeholder. New layer creation lands here in a later stage." }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

pub(crate) fn render_objects(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_state(state, "Load the embedded demo before browsing objects.");
    };

    let objects = collect_objects(session);

    rsx! {
        div { class: "mobile-screen",
            {render_detail_header("Object Library", state)}
            div { class: "mobile-page-body",
                div { class: "mobile-search",
                    input {
                        placeholder: "Search objects...",
                        value: "",
                        readonly: true,
                    }
                }
                div { class: "mobile-object-grid",
                    for entry in objects.iter() {
                        button {
                            key: "mobile-object-{entry.object_id}",
                            class: if snapshot.selected_object == Some(entry.object_id) { "mobile-object-card active" } else { "mobile-object-card" },
                            onclick: {
                                let entry = entry.clone();
                                move |_| {
                                    let mut state = state.write();
                                    state.active_layer = entry.layer_index;
                                    state.selected_object = Some(entry.object_id);
                                }
                            },
                            span {
                                class: "mobile-object-icon",
                                style: object_icon_style(&entry.shape),
                            }
                            span { class: "mobile-object-label", "{entry.name}" }
                        }
                    }
                }
                div { class: "mobile-inline-actions wide",
                    button {
                        onclick: move |_| create_object_on_first_object_layer(&mut state.write(), ObjectShape::Rectangle),
                        "Add Rect"
                    }
                    button {
                        onclick: move |_| create_object_on_first_object_layer(&mut state.write(), ObjectShape::Point),
                        "Add Point"
                    }
                    button {
                        onclick: move |_| delete_selected_object(&mut state.write()),
                        "Delete"
                    }
                    button {
                        onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                        "Canvas"
                    }
                }
                if let Some((object, _layer_index)) = selected_object_view(session, snapshot.selected_object, snapshot.active_layer) {
                    div { class: "mobile-card",
                        strong { "Selected Object: {object.name} (ID {object.id})" }
                        p { {format!("Global Coordinates: (X: {:.0}, Y: {:.0})", object.x, object.y)} }
                    }
                    div { class: "mobile-inline-actions wide",
                        button { onclick: move |_| nudge_selected_object(&mut state.write(), -16.0, 0.0), "Left" }
                        button { onclick: move |_| nudge_selected_object(&mut state.write(), 16.0, 0.0), "Right" }
                        button { onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, -16.0), "Up" }
                        button { onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, 16.0), "Down" }
                    }
                    label { class: "mobile-field",
                        span { "Name" }
                        input {
                            value: object.name.clone(),
                            onchange: move |event| rename_selected_object(&mut state.write(), event.value()),
                        }
                    }
                }
                div { class: "mobile-placeholder-card compact",
                    strong { "Attached Scripts & Events" }
                    p { "UI placeholder for future script bindings, triggers, and event metadata." }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

pub(crate) fn render_settings(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-screen",
            {render_detail_header("App Settings", state)}
            div { class: "mobile-page-body",
                div { class: "mobile-card",
                    div { class: "mobile-section-label", "Session Actions" }
                    div { class: "mobile-inline-actions wide",
                        button {
                            onclick: move |_| {
                                let mut state = state.write();
                                load_sample(&mut state);
                                state.mobile_screen = MobileScreen::Editor;
                            },
                            "Load Demo"
                        }
                        button { onclick: move |_| save_document(&mut state.write()), "Save" }
                        button {
                            onclick: move |_| {
                                let mut state = state.write();
                                if state.session.as_mut().is_some_and(EditorSession::undo) {
                                    state.status = "Undo applied.".to_string();
                                } else {
                                    state.status = "Nothing to undo.".to_string();
                                }
                            },
                            "Undo"
                        }
                        button {
                            onclick: move |_| {
                                let mut state = state.write();
                                if state.session.as_mut().is_some_and(EditorSession::redo) {
                                    state.status = "Redo applied.".to_string();
                                } else {
                                    state.status = "Nothing to redo.".to_string();
                                }
                            },
                            "Redo"
                        }
                    }
                }
                div { class: "mobile-card",
                    div { class: "mobile-section-label", "View" }
                    div { class: "mobile-settings-row",
                        span { "Zoom" }
                        div { class: "mobile-inline-actions",
                            button { onclick: move |_| adjust_zoom(&mut state.write(), -25), "-" }
                            span { "{snapshot.zoom_percent}%" }
                            button { onclick: move |_| adjust_zoom(&mut state.write(), 25), "+" }
                        }
                    }
                    div { class: "mobile-settings-row placeholder",
                        span { "Grid Settings" }
                        span { "UI placeholder" }
                    }
                    div { class: "mobile-settings-row placeholder",
                        span { "Theme" }
                        span { "Dark" }
                    }
                }
                div { class: "mobile-card",
                    div { class: "mobile-section-label", "Diagnostics" }
                    p { class: "mobile-status-copy", "{snapshot.status}" }
                    {render_log_path()}
                }
                div { class: "mobile-placeholder-card",
                    strong { "Export Settings" }
                    p { "JSON / XML / PNG toggles are represented here as planned UI, but export configuration is not wired yet." }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

#[cfg(target_os = "android")]
fn render_log_path() -> Element {
    let log_path = log_path().unwrap_or_default();
    rsx! { p { class: "mobile-status-copy", "Log file: {log_path}" } }
}

#[cfg(not(target_os = "android"))]
fn render_log_path() -> Element {
    rsx! {}
}

fn collect_objects(session: &EditorSession) -> Vec<MobileObjectSummary> {
    let mut objects = Vec::new();
    for (layer_index, layer) in session.document().map.layers.iter().enumerate() {
        if let Some(object_layer) = layer.as_object() {
            for object in &object_layer.objects {
                objects.push(MobileObjectSummary {
                    layer_index,
                    object_id: object.id,
                    name: object.name.clone(),
                    shape: object.shape.clone(),
                });
            }
        }
    }
    objects
}

fn create_object_on_first_object_layer(state: &mut AppState, shape: ObjectShape) {
    let object_layer_index = match state.session.as_ref() {
        Some(session) => session
            .document()
            .map
            .layers
            .iter()
            .enumerate()
            .find(|(_, layer)| layer.as_object().is_some())
            .map(|(index, _)| index),
        None => {
            state.status = "Load a map first.".to_string();
            return;
        }
    };

    let Some(object_layer_index) = object_layer_index else {
        state.status = "No object layer is available yet.".to_string();
        return;
    };

    state.active_layer = object_layer_index;
    create_object(state, shape);
}
