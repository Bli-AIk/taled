use dioxus::prelude::*;
use std::sync::LazyLock;

use crate::app_state::{AppState, MobileScreen};
use base64::Engine;

static DASHBOARD_FOREST: LazyLock<String> =
    LazyLock::new(|| review_data_url(include_bytes!("../../../assets/review/dashboard-forest.png")));
static DASHBOARD_DUNGEON: LazyLock<String> =
    LazyLock::new(|| review_data_url(include_bytes!("../../../assets/review/dashboard-dungeon.png")));
static DASHBOARD_CASTLE: LazyLock<String> =
    LazyLock::new(|| review_data_url(include_bytes!("../../../assets/review/dashboard-castle.png")));
static DASHBOARD_DESERT: LazyLock<String> =
    LazyLock::new(|| review_data_url(include_bytes!("../../../assets/review/dashboard-desert.png")));
static DASHBOARD_WINTER: LazyLock<String> =
    LazyLock::new(|| review_data_url(include_bytes!("../../../assets/review/dashboard-winter.png")));

pub(crate) fn render_review_shell(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-shell review-shell",
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

fn render_dashboard(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let projects = [
        ("forest", "Green Forest Level 1", "Last edited: Oct 26, 2023 • 1.2 MB"),
        ("dungeon", "Dungeon Interior", "Last edited: Oct 25, 2023 • 850 KB"),
        ("castle", "Castle Tileset", "Last edited: Oct 24, 2023 • 2.5 MB"),
        ("desert", "Desert Outpost", "Last edited: Oct 23, 2023 • 980 KB"),
        ("winter", "Winter Village", "Last edited: Oct 22, 2023 • 1.5 MB"),
    ];

    rsx! {
        div { class: "review-page",
            {review_top_bar("Project Dashboard", None, Some("+ New Project"), state)}
            div { class: "review-body",
                button {
                    class: "review-create-project",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                    {review_plus_icon("review-plus-icon")}
                    span { "Create New Project" }
                }
                div { class: "review-project-list-panel",
                    for (kind, title, meta) in projects {
                        button {
                            key: "{kind}",
                            class: "review-project-row",
                            onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                            img {
                                class: "review-project-thumb",
                                src: dashboard_thumb_src(kind),
                                alt: "{title} thumbnail",
                            }
                            div { class: "review-project-copy",
                                div { class: "review-project-title", "{title}" }
                                div { class: "review-project-meta", "{meta}" }
                            }
                        }
                    }
                }
            }
            {review_nav(snapshot, state, true)}
        }
    }
}

fn dashboard_thumb_src(kind: &str) -> &'static str {
    match kind {
        "forest" => DASHBOARD_FOREST.as_str(),
        "dungeon" => DASHBOARD_DUNGEON.as_str(),
        "castle" => DASHBOARD_CASTLE.as_str(),
        "desert" => DASHBOARD_DESERT.as_str(),
        "winter" => DASHBOARD_WINTER.as_str(),
        _ => "",
    }
}

fn review_data_url(bytes: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

fn render_editor(snapshot: &AppState, state: Signal<AppState>) -> Element {
    let tools = [
        ("Select", MobileScreen::Editor, true),
        ("Brush", MobileScreen::Editor, false),
        ("Eraser", MobileScreen::Editor, false),
        ("Bucket Fill", MobileScreen::Editor, false),
    ];

    rsx! {
        div { class: "review-page review-editor-page",
            {review_top_bar("Tile Map Editor Main Canvas", None, None, state)}
            div { class: "review-editor-canvas",
                div { class: "review-map-surface",
                    div { class: "review-map-grass a" }
                    div { class: "review-map-grass b" }
                    div { class: "review-map-path" }
                    div { class: "review-map-wall left" }
                    div { class: "review-map-wall right" }
                    div { class: "review-map-shadow" }
                }
                div { class: "review-dpad",
                    span { class: "up", "^" }
                    span { class: "left", "<" }
                    span { class: "center", "Q" }
                    span { class: "right", ">" }
                    span { class: "down", "v" }
                }
                div { class: "review-layer-float",
                    div { class: "review-layer-float-title", "Layers" }
                    div { class: "review-layer-float-item",
                        span { class: "review-eye on", "o" }
                        span { "Foreground" }
                        span { class: "review-menu-glyph", "≡" }
                    }
                    div { class: "review-layer-float-item",
                        span { class: "review-eye on", "o" }
                        span { "Obstacles" }
                        span { class: "review-menu-glyph", "≡" }
                    }
                    div { class: "review-layer-float-item muted",
                        span { class: "review-eye off", "o" }
                        span { "Background" }
                        span { class: "review-menu-glyph", "≡" }
                    }
                }
            }
            div { class: "review-editor-toolbar",
                div { class: "review-tool-row",
                    for (label, _screen, active) in tools {
                        div {
                            key: "{label}",
                            class: if active { "review-tool active" } else { "review-tool" },
                            div { class: "review-tool-icon" }
                            span { "{label}" }
                        }
                    }
                }
                div { class: "review-tile-strip",
                    div { class: "review-tile-chip selected grass" }
                    div { class: "review-tile-chip path" }
                    div { class: "review-tile-chip sand" }
                    div { class: "review-tile-chip stone" }
                    div { class: "review-tile-chip fence" }
                    div { class: "review-tile-chip tree" }
                    div { class: "review-tile-chip tree2" }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_tilesets(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "review-page",
            {review_top_bar("Tileset Property Editor", Some("Back"), Some("Done"), state)}
            div { class: "review-body review-section-stack",
                div { class: "review-section-title", "Sprite Sheet View" }
                div { class: "review-tileset-sheet",
                    for index in 0..24 {
                        div {
                            key: "tile-{index}",
                            class: if index == 14 { "review-sheet-cell active" } else { "review-sheet-cell" },
                        }
                    }
                }
                div { class: "review-section-title with-gap", "Selected Tile: ID 14" }
                {review_input_row("Name:", "Wooden Chest")}
                {review_input_row("Type:", "Object")}
                div { class: "review-section-title with-gap", "Custom Properties" }
                div { class: "review-setting-row",
                    span { "isSolid" }
                    div { class: "review-toggle on", div { class: "knob" } }
                }
                div { class: "review-setting-row",
                    span { "damage" }
                    div { class: "review-stepper",
                        button { "-" }
                        span { "10" }
                        button { "+" }
                    }
                }
                button { class: "review-link-button", "+ Add Property" }
                div { class: "review-section-title with-gap", "Collision Editor" }
                div { class: "review-collision-row",
                    div { class: "review-collision-tools",
                        div { class: "review-collision-tool active", "Box" }
                        div { class: "review-collision-tool", "Polygon" }
                    }
                    div { class: "review-collision-actions",
                        div { class: "review-trash", "Clear" }
                        div { class: "review-collision-preview" }
                    }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_layers(snapshot: &AppState, state: Signal<AppState>) -> Element {
    let layers = [
        ("ui", "UI", "100%", true, true),
        ("decor", "Top Decorations", "85%", true, true),
        ("foreground", "Foreground", "100%", true, true),
        ("obstacles", "Obstacles", "50%", false, true),
        ("ground", "Ground", "85%", true, true),
        ("background", "Background", "30%", false, true),
    ];

    rsx! {
        div { class: "review-page",
            {review_top_bar("Layer Manager", Some("Back"), Some("Done"), state)}
            div { class: "review-body review-list",
                for (kind, name, opacity, visible, locked) in layers {
                    div {
                        key: "{name}",
                        class: "review-layer-row",
                        span { class: "review-menu-glyph", "≡" }
                        div { class: "review-layer-thumb {kind}" }
                        div { class: "review-layer-name", "{name}" }
                        span { class: if visible { "review-eye on" } else { "review-eye off" }, "o" }
                        span { class: if locked { "review-lock on" } else { "review-lock off" }, "u" }
                        div { class: "review-opacity",
                            span { "Opacity" }
                            span { "{opacity}" }
                            div { class: "review-slider-track",
                                div { class: "review-slider-fill", style: "width:{opacity};" }
                                div { class: "review-slider-knob", style: "left:calc({opacity} - 10px);" }
                            }
                        }
                    }
                }
                div { class: "review-empty-row" }
                button { class: "review-secondary-button", "Add Layer" }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_objects(snapshot: &AppState, state: Signal<AppState>) -> Element {
    let objects = [
        ("villager", "NPC: Villager", false),
        ("chest", "Object: Wooden Chest", true),
        ("portal", "Portal: Dungeon Entrance", false),
        ("slime", "Enemy: Slime", false),
        ("potion", "Item: Potion", false),
        ("flag", "Trigger: Event Marker", false),
    ];

    rsx! {
        div { class: "review-page",
            {review_top_bar("Object Library", None, None, state)}
            div { class: "review-body review-section-stack",
                div { class: "review-search-bar",
                    span { class: "review-search-icon", "o" }
                    span { class: "muted", "Search objects..." }
                }
                div { class: "review-object-grid",
                    for (kind, label, active) in objects {
                        div {
                            key: "{kind}",
                            class: if active { "review-object-card active" } else { "review-object-card" },
                            div { class: "review-object-art {kind}" }
                            div { class: "review-object-card-label", "{label}" }
                        }
                    }
                    div { class: "review-object-card ghost" }
                    div { class: "review-object-card ghost" }
                }
                div { class: "review-info-card",
                    div { class: "review-info-title", "Selected Object: Wooden Chest (ID 45)" }
                    div { class: "review-info-meta", "Global Coordinates: (X: 52, Y: 18)" }
                }
                div { class: "review-info-card" ,
                    div { class: "review-info-title", "Attached Scripts & Events" }
                    div { class: "review-script-row", "OnInteract: open_chest.lua" }
                    div { class: "review-script-row", "OnDestroy: loot_table.lua" }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_settings(_snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "review-page",
            {review_top_bar("App Settings", Some("Back"), Some("Done"), state)}
            div { class: "review-body review-section-stack",
                div { class: "review-caption", "Grid Settings" }
                div { class: "review-settings-card",
                    {review_setting_with_chip("Grid Color", "#CCCCCC")}
                    {review_slider_row("Grid Size", "32x32 px")}
                    div { class: "review-setting-row",
                        span { "Snapping" }
                        div { class: "review-toggle on", div { class: "knob" } }
                    }
                }
                div { class: "review-caption", "Theme" }
                div { class: "review-segmented",
                    button { class: "active", "Dark" }
                    button { "Light" }
                    button { "System" }
                }
                div { class: "review-caption", "Auto-save" }
                div { class: "review-settings-card single",
                    div { class: "review-setting-row",
                        span { "Frequency" }
                        span { class: "muted", "Every 5 mins" }
                    }
                }
                div { class: "review-caption", "Export Settings" }
                div { class: "review-settings-card",
                    div { class: "review-setting-row", span { "JSON" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "XML" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "PNG" } div { class: "review-toggle on", div { class: "knob" } } }
                }
                div { class: "review-caption", "Cloud Sync" }
                button { class: "review-sync-button", "Sync Now" }
                div { class: "review-sync-meta", "Last synced: Today, 4:30 PM" }
                div { class: "review-settings-card single",
                    div { class: "review-setting-row",
                        span { "Automatic Sync" }
                        div { class: "review-toggle on", div { class: "knob" } }
                    }
                }
            }
        }
    }
}

fn review_top_bar(
    title: &'static str,
    left: Option<&'static str>,
    right: Option<&'static str>,
    mut state: Signal<AppState>,
) -> Element {
    rsx! {
        div { class: "review-header",
            if let Some(label) = left {
                button {
                    class: "review-header-action left",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                    "{label}"
                }
            } else {
                div { class: "review-header-spacer" }
            }
            h1 { "{title}" }
            if let Some(label) = right {
                button {
                    class: "review-header-action right",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                    if label.starts_with("+ ") {
                        {review_plus_icon("review-header-plus")}
                        span { class: "review-header-link-label", "{label.trim_start_matches(\"+ \")}" }
                    } else {
                        "{label}"
                    }
                }
            } else {
                div { class: "review-header-spacer" }
            }
        }
    }
}

fn review_nav(snapshot: &AppState, state: Signal<AppState>, dashboard_variant: bool) -> Element {
    rsx! {
        div { class: if dashboard_variant { "review-bottom-nav dashboard" } else { "review-bottom-nav editor" },
            if dashboard_variant {
                {review_nav_button(snapshot, state, MobileScreen::Dashboard, "Projects")}
                div { class: "review-nav-item muted", "Assets" }
                {review_nav_button(snapshot, state, MobileScreen::Settings, "Settings")}
            } else {
                {review_nav_button(snapshot, state, MobileScreen::Tilesets, "Tilesets")}
                {review_nav_button(snapshot, state, MobileScreen::Layers, "Layers")}
                {review_nav_button(snapshot, state, MobileScreen::Objects, "Objects")}
                {review_nav_button(snapshot, state, MobileScreen::Settings, "Settings")}
            }
        }
    }
}

fn review_nav_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    screen: MobileScreen,
    label: &'static str,
) -> Element {
    let icon_class = match label {
        "Projects" => "review-nav-icon projects",
        "Assets" => "review-nav-icon assets",
        "Tilesets" => "review-nav-icon tilesets",
        "Layers" => "review-nav-icon layers",
        "Objects" => "review-nav-icon objects",
        "Settings" => "review-nav-icon settings",
        _ => "review-nav-icon",
    };

    rsx! {
        button {
            class: if snapshot.mobile_screen == screen { "review-nav-item active" } else { "review-nav-item" },
            onclick: move |_| state.write().mobile_screen = screen.clone(),
            div { class: "{icon_class}" , {review_nav_icon(label)} }
            span { "{label}" }
        }
    }
}

fn review_plus_icon(class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 5v14" }
            path { d: "M5 12h14" }
        }
    }
}

fn review_nav_icon(label: &'static str) -> Element {
    match label {
        "Projects" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M8 6h13" }
                path { d: "M8 12h13" }
                path { d: "M8 18h13" }
                path { d: "M3 6h.01" }
                path { d: "M3 12h.01" }
                path { d: "M3 18h.01" }
            }
        },
        "Assets" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
            }
        },
        "Tilesets" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
            }
        },
        "Layers" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 3 3 8l9 5 9-5-9-5z" }
                path { d: "m3 12 9 5 9-5" }
                path { d: "m3 16 9 5 9-5" }
            }
        },
        "Objects" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 2v20" }
                path { d: "M2 12h20" }
                circle { cx: "12", cy: "12", r: "5" }
            }
        },
        "Settings" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "12", cy: "12", r: "3" }
                path { d: "M19.4 15a1.7 1.7 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-.4-1 1.7 1.7 0 0 0-1-.6 1.7 1.7 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1-.4H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1-.4 1.7 1.7 0 0 0 .6-1 1.7 1.7 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.7 1.7 0 0 0 9 4.6c.38-.08.72-.28 1-.6a1.7 1.7 0 0 0 .4-1V3a2 2 0 1 1 4 0v.09c0 .38.14.74.4 1 .28.32.62.52 1 .6a1.7 1.7 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.7 1.7 0 0 0 19.4 9c.08.38.28.72.6 1 .26.26.62.4 1 .4H21a2 2 0 1 1 0 4h-.09c-.38 0-.74.14-1 .4-.32.28-.52.62-.6 1z" }
            }
        },
        _ => rsx! { span {} },
    }
}

fn review_input_row(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "review-input-row",
            span { class: "label", "{label}" }
            div { class: "review-input-box", "{value}" }
        }
    }
}

fn review_setting_with_chip(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "review-setting-row",
            span { "{label}" }
            div { class: "review-color-chip",
                div { class: "swatch" }
                span { "{value}" }
            }
        }
    }
}

fn review_slider_row(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "review-setting-row slider",
            span { "{label}" }
            div { class: "review-slider-track wide",
                div { class: "review-slider-fill", style: "width:48%;" }
                div { class: "review-slider-knob", style: "left:calc(48% - 10px);" }
            }
            span { class: "muted", "{value}" }
        }
    }
}
