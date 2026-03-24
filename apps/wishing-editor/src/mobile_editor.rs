use dioxus::prelude::*;
use wishing_core::EditorSession;

use crate::{
    app_state::{AppState, MobileScreen, PaletteTile, Tool},
    mobile_shell::{render_bottom_nav, render_detail_header, render_missing_state},
    session_ops::adjust_zoom,
    ui_canvas::render_canvas,
    ui_inspector::collect_palette,
    ui_visuals::palette_tile_style,
};

pub(crate) fn render_editor(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let palette = snapshot
        .session
        .as_ref()
        .map(|session| collect_palette(session.document()))
        .unwrap_or_default();
    let title = crate::mobile_shell::document_title(snapshot);

    rsx! {
        div { class: "mobile-screen mobile-editor-screen",
            div { class: "mobile-page-header mobile-editor-header",
                button {
                    class: "mobile-inline-action",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Dashboard,
                    "Projects"
                }
                div { class: "mobile-editor-title",
                    h1 { "Tile Map Editor" }
                    p { "{title}" }
                }
                button {
                    class: "mobile-inline-action",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Layers,
                    "Layers"
                }
            }
            div { class: "mobile-editor-canvas-shell",
                {render_canvas(snapshot, state)}
                {render_layer_overlay(snapshot, state)}
                {render_dpad(snapshot, state)}
            }
            div { class: "mobile-tool-tray",
                div { class: "mobile-tool-row",
                    {tool_chip(snapshot, state, Tool::Select, "Select")}
                    {tool_chip(snapshot, state, Tool::Paint, "Brush")}
                    {tool_chip(snapshot, state, Tool::Erase, "Eraser")}
                    {tool_chip(snapshot, state, Tool::AddRectangle, "Rect")}
                    {tool_chip(snapshot, state, Tool::AddPoint, "Point")}
                }
                div { class: "mobile-inline-actions wide",
                    button { onclick: move |_| adjust_zoom(&mut state.write(), -25), "Zoom -" }
                    button { onclick: move |_| adjust_zoom(&mut state.write(), 25), "Zoom +" }
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
                if palette.is_empty() {
                    div { class: "mobile-placeholder-card compact",
                        strong { "Tileset tray" }
                        p { "Load the embedded sample to browse tiles." }
                    }
                } else {
                    div { class: "mobile-tile-strip",
                        for tile in palette.into_iter().take(24) {
                            button {
                                key: "mobile-tile-{tile.gid}",
                                class: if snapshot.selected_gid == tile.gid { "mobile-tile-chip active" } else { "mobile-tile-chip" },
                                style: snapshot
                                    .session
                                    .as_ref()
                                    .map(|session| palette_tile_style(session.document(), &snapshot.image_cache, &tile))
                                    .unwrap_or_default(),
                                onclick: move |_| {
                                    let mut state = state.write();
                                    state.selected_gid = tile.gid;
                                    state.status = format!("Selected gid {}.", tile.gid);
                                },
                            }
                        }
                    }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

pub(crate) fn render_tilesets(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_state(state, "Load the embedded demo before editing tilesets.");
    };

    let palette = collect_palette(session.document());
    let selected_gid = snapshot.selected_gid;
    let selected_summary = session
        .document()
        .map
        .tile_reference_for_gid(selected_gid)
        .map(|reference| {
            format!(
                "Tile ID {} from {}",
                selected_gid, reference.tileset.tileset.name
            )
        })
        .unwrap_or_else(|| "Choose a tile from the sheet below.".to_string());

    rsx! {
        div { class: "mobile-screen",
            {render_detail_header("Tileset Property Editor", state)}
            div { class: "mobile-page-body",
                div { class: "mobile-card",
                    div { class: "mobile-section-label", "Sprite Sheet View" }
                    div { class: "mobile-tileset-grid",
                        for tile in palette {
                            button {
                                key: "tileset-view-{tile.gid}",
                                class: if selected_gid == tile.gid { "mobile-tileset-cell active" } else { "mobile-tileset-cell" },
                                style: palette_tile_style(session.document(), &snapshot.image_cache, &tile),
                                onclick: move |_| {
                                    let mut state = state.write();
                                    state.selected_gid = tile.gid;
                                    state.status = format!("Selected gid {}.", tile.gid);
                                },
                            }
                        }
                    }
                }

                div { class: "mobile-card",
                    div { class: "mobile-section-label", "Selected Tile" }
                    div { class: "mobile-selected-preview",
                        div {
                            class: "mobile-selected-preview-art",
                            style: selected_tile_style(snapshot, session, selected_gid),
                        }
                        div { class: "mobile-selected-preview-copy",
                            strong { "{selected_summary}" }
                            p { "Tile property editing and collision authoring stay on this page once implemented." }
                        }
                    }
                }

                div { class: "mobile-placeholder-card",
                    strong { "Custom Properties" }
                    p { "Stage-2 placeholder: property rows, typed editors, and collision tools will live here." }
                }
            }
            {render_bottom_nav(snapshot, state)}
        }
    }
}

fn tool_chip(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    tool: Tool,
    label: &'static str,
) -> Element {
    let class = if snapshot.tool == tool { "active" } else { "" };
    rsx! {
        button {
            class: class,
            onclick: move |_| state.write().tool = tool.clone(),
            "{label}"
        }
    }
}

fn render_layer_overlay(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return rsx! {};
    };

    rsx! {
        div { class: "mobile-layer-overlay",
            div { class: "mobile-overlay-header",
                strong { "Layers" }
                button {
                    class: "mobile-inline-action subtle",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Layers,
                    "Open"
                }
            }
            for (index, layer) in session.document().map.layers.iter().enumerate().take(3) {
                div {
                    key: "overlay-layer-{index}",
                    class: if snapshot.active_layer == index { "mobile-overlay-layer active" } else { "mobile-overlay-layer" },
                    button {
                        onclick: move |_| {
                            let mut state = state.write();
                            state.active_layer = index;
                            state.selected_object = None;
                        },
                        strong { "{layer.name()}" }
                        span { "{crate::mobile_shell::layer_kind_label(layer)}" }
                    }
                    span { if layer.visible() { "On" } else { "Off" } }
                }
            }
        }
    }
}

fn render_dpad(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-dpad",
            button { class: "up", onclick: move |_| state.write().pan_y -= 32, "^" }
            button { class: "left", onclick: move |_| state.write().pan_x -= 32, "<" }
            button { class: "center", onclick: move |_| state.write().mobile_screen = MobileScreen::Tilesets, "{snapshot.zoom_percent}%" }
            button { class: "right", onclick: move |_| state.write().pan_x += 32, ">" }
            button { class: "down", onclick: move |_| state.write().pan_y += 32, "v" }
        }
    }
}

fn selected_tile_style(snapshot: &AppState, session: &EditorSession, selected_gid: u32) -> String {
    let Some(reference) = session.document().map.tile_reference_for_gid(selected_gid) else {
        return String::new();
    };

    palette_tile_style(
        session.document(),
        &snapshot.image_cache,
        &PaletteTile {
            gid: selected_gid,
            tileset_index: reference.tileset_index,
            local_id: reference.local_id,
        },
    )
}
