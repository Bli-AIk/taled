use std::{collections::BTreeMap, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
use base64::Engine;
use dioxus::prelude::*;
use taled_core::{EditorDocument, ObjectShape};

#[cfg(not(test))]
fn eval_js(js: &str) {
    dioxus::document::eval(js);
}

#[cfg(test)]
fn eval_js(_js: &str) {}

use crate::{
    app_state::{
        AppState, TileSelectionRegion, Tool, is_tile_selection_tool, selection_bounds,
        selection_cells_are_rectangular, selection_cells_from_region, selection_region_from_cells,
        shape_fill_cells,
    },
    edit_ops::{apply_cell_tool, clear_tile_selection_immediately},
    platform::log,
    touch_ops::{
        cell_from_surface, handle_touch_pointer_cancel, handle_touch_pointer_down,
        handle_touch_pointer_move, handle_touch_pointer_up, should_ignore_synthetic_click,
    },
    ui_visuals::object_overlay_style,
};

const TILE_SELECTION_FADE_DURATION: Duration = Duration::from_millis(170);

pub(crate) fn render_canvas(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return rsx! {
            div { class: "canvas-host",
                div { class: "empty-state", "No map loaded yet." }
            }
        };
    };

    let document = session.document();
    let map = &document.map;
    let visible_bounds = visible_cell_bounds(snapshot, map);
    let zoom = snapshot.zoom_percent as f32 / 100.0;
    let canvas_style = format!(
        "width:{}px;height:{}px;transform:translate3d({}px, {}px, 0) scale({zoom});",
        map.total_pixel_width(),
        map.total_pixel_height(),
        snapshot.pan_x,
        snapshot.pan_y
    );
    let canvas_class = if snapshot.camera_transition_active {
        "canvas camera-transition"
    } else {
        "canvas"
    };
    let shape_fill_preview = if snapshot.tool == Tool::ShapeFill {
        snapshot
            .shape_fill_preview
            .map(|preview| build_shape_fill_preview(document, snapshot, preview))
    } else {
        None
    };
    let tile_selection_overlay = active_tile_selection_overlay(document, snapshot);
    let has_tile_selection_overlay = tile_selection_overlay.is_some();
    let tile_selection_transfer_preview =
        active_tile_selection_transfer_preview(document, snapshot);
    let render_objects_live = matches!(
        snapshot.tool,
        Tool::Select | Tool::AddRectangle | Tool::AddPoint
    );
    let show_cached_object_layers =
        !render_objects_live && snapshot.flat_object_layers_data_url.is_some();
    let show_live_active_tile_overlay = snapshot.touch_edit_batch_active
        || (snapshot.active_tile_layer_separated && snapshot.active_tile_layer_data_url.is_none());
    let live_active_tile_styles = if show_live_active_tile_overlay {
        collect_visible_tile_styles(document, snapshot)
    } else {
        Vec::new()
    };

    rsx! {
        div {
            class: "canvas-host",
            onmounted: move |event| {
                let mut state = state;
                async move {
                    if let Ok(rect) = event.get_client_rect().await {
                        log(format!(
                            "touch:host-rect origin=({:.1},{:.1}) size=({:.1},{:.1})",
                            rect.origin.x,
                            rect.origin.y,
                            rect.size.width,
                            rect.size.height,
                        ));
                        let mut state = state.write();
                        state.canvas_stage_client_origin = Some((rect.origin.x, rect.origin.y));
                        state.canvas_host_size = Some((rect.size.width, rect.size.height));
                        center_canvas_if_needed(&mut state, rect.size.width, rect.size.height);
                    }
                    if let Ok(scroll) = event.get_scroll_offset().await {
                        log(format!(
                            "touch:host-scroll offset=({:.1},{:.1})",
                            scroll.x, scroll.y,
                        ));
                        state.write().canvas_host_scroll_offset = (scroll.x, scroll.y);
                    }
                }
            },
            onscroll: move |event| {
                let mut state = state.write();
                let scroll_left = event.scroll_left();
                let scroll_top = event.scroll_top();
                log(format!(
                    "touch:host-scroll offset=({scroll_left:.1},{scroll_top:.1}) size=({},{}) client=({},{})",
                    event.scroll_width(),
                    event.scroll_height(),
                    event.client_width(),
                    event.client_height(),
                ));
                state.canvas_host_scroll_offset = (scroll_left, scroll_top);
            },
            div {
                    class: "canvas-stage",
                onmounted: move |event| {
                    let mut state = state;
                    async move {
                        if let Ok(rect) = event.get_client_rect().await {
                            log(format!(
                                "touch:stage-rect origin=({:.1},{:.1}) size=({:.1},{:.1})",
                                rect.origin.x,
                                rect.origin.y,
                                rect.size.width,
                                rect.size.height,
                            ));
                            center_canvas_if_needed(&mut state.write(), rect.size.width, rect.size.height);
                        }
                    }
                },
                onclick: move |event| {
                    let mut state = state.write();
                    if should_ignore_synthetic_click(&mut state) {
                        return;
                    }
                    handle_canvas_click(
                        &mut state,
                        event.data().element_coordinates().x,
                        event.data().element_coordinates().y,
                    );
                },
                onpointerdown: move |event| handle_touch_pointer_down(&mut state.write(), event),
                onpointermove: move |event| handle_touch_pointer_move(&mut state.write(), event),
                onpointerup: move |event| handle_touch_pointer_up(&mut state.write(), event),
                onpointercancel: move |event| handle_touch_pointer_cancel(&mut state.write(), event),
                div {
                    class: canvas_class,
                    style: canvas_style,
                    ontransitionend: move |_| state.write().camera_transition_active = false,
                    img {
                        id: "taled-fc",
                        class: "canvas-flat-layer",
                        alt: "",
                        style: "left:0;top:0;width:{map.total_pixel_width()}px;height:{map.total_pixel_height()}px;",
                    }
                    if snapshot.active_tile_layer_separated && !snapshot.touch_edit_batch_active {
                        img {
                            id: "taled-ac",
                            class: "canvas-flat-layer canvas-active-layer",
                            alt: "",
                            style: "left:0;top:0;width:{map.total_pixel_width()}px;height:{map.total_pixel_height()}px;",
                        }
                    }
                    for (x, y, style) in live_active_tile_styles.iter() {
                        div {
                            key: "tile-{x}-{y}",
                            class: "tile-sprite",
                            style: "{style}",
                        }
                    }
                    if show_cached_object_layers {
                        if let (Some(data_url), Some(style)) = (
                            snapshot.flat_object_layers_data_url.as_ref(),
                            flat_tile_cache_style(snapshot.flat_object_layers_cell_bounds, map),
                        ) {
                            img {
                                class: "canvas-flat-layer canvas-object-layer",
                                src: "{data_url}",
                                alt: "",
                                style: "{style}",
                            }
                        }
                    }

                    if render_objects_live || !show_cached_object_layers {
                        for (layer_index, layer) in map.layers.iter().enumerate() {
                            if let Some(object_layer) = layer.as_object() {
                                if object_layer.visible {
                                    for object in &object_layer.objects {
                                        if object_intersects_visible_bounds(
                                            object,
                                            visible_bounds,
                                            map.tile_width,
                                            map.tile_height,
                                        ) {
                                            div {
                                                key: "object-{layer_index}-{object.id}",
                                                class: object_class(snapshot.selected_object, object.id, &object.shape),
                                                style: object_overlay_style(
                                                    object,
                                                    snapshot.tool == Tool::Select,
                                                    snapshot.selected_object == Some(object.id),
                                                    zoom,
                                                ),
                                                onclick: {
                                                    let object_id = object.id;
                                                    move |_| {
                                                        let mut state = state.write();
                                                        if should_ignore_synthetic_click(&mut state) {
                                                            return;
                                                        }
                                                        if state.tool != Tool::Select {
                                                            state.status = "Switch to Select before choosing objects.".to_string();
                                                            return;
                                                        }
                                                        state.active_layer = layer_index;
                                                        state.selected_object = Some(object_id);
                                                        state.tile_selection = None;
                                                        state.tile_selection_cells = None;
                                                        state.tile_selection_preview = None;
                                                        state.tile_selection_preview_cells = None;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    {shape_fill_preview.as_ref().map(|preview| rsx! {
                        for tile in &preview.tiles {
                            div {
                                key: "shape-fill-preview-{tile.x}-{tile.y}",
                                class: if tile.fallback {
                                    "shape-fill-preview-tile fallback"
                                } else {
                                    "tile-preview shape-fill-preview-tile"
                                },
                                style: "{tile.style}",
                            }
                        }
                        div {
                            class: "shape-fill-preview-frame",
                            style: "{preview.frame_style}",
                        }
                    })}

                    {tile_selection_overlay.as_ref().map(|overlay| rsx! {
                        if overlay.irregular {
                            Fragment {
                                div {
                                    class: if overlay.closing {
                                        "tile-selection-region-cells closing"
                                    } else if overlay.preview {
                                        "tile-selection-region-cells preview"
                                    } else {
                                        "tile-selection-region-cells"
                                    },
                                    for (index, cell_style) in overlay.cell_styles.iter().enumerate() {
                                        div {
                                            key: "tile-selection-cell-{index}",
                                            class: "tile-selection-cell-fragment",
                                            style: "{cell_style}",
                                        }
                                    }
                                }
                                div {
                                    class: if overlay.closing {
                                        "tile-selection-irregular-bounds closing"
                                    } else if overlay.preview {
                                        "tile-selection-irregular-bounds preview"
                                    } else {
                                        "tile-selection-irregular-bounds"
                                    },
                                    style: "{overlay.region_style}",
                                    if overlay.show_irregular_handles {
                                        for handle in &overlay.handles {
                                            div {
                                                key: "tile-selection-irregular-handle-{handle.position}",
                                                class: "tile-selection-handle ghost {handle.position}",
                                                style: "{handle.style}",
                                                div { class: "tile-selection-handle-dot ghost" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: if overlay.closing {
                                    "tile-selection-region closing"
                                } else if overlay.preview {
                                    "tile-selection-region preview"
                                } else {
                                    "tile-selection-region"
                                },
                                style: "{overlay.region_style}",
                                div { class: "tile-selection-frame" }
                                if overlay.show_handles {
                                    for handle in &overlay.handles {
                                        div {
                                            key: "tile-selection-handle-{handle.position}",
                                            class: "tile-selection-handle {handle.position}",
                                            style: "{handle.style}",
                                            div { class: "tile-selection-handle-dot" }
                                        }
                                    }
                                }
                            }
                        }
                    })}

                    {tile_selection_transfer_preview.as_ref().map(|preview| rsx! {
                        for tile in &preview.tiles {
                            div {
                                key: "tile-selection-transfer-preview-{tile.x}-{tile.y}",
                                class: if tile.fallback {
                                    "shape-fill-preview-tile tile-selection-transfer-preview-tile fallback"
                                } else {
                                    "tile-preview shape-fill-preview-tile tile-selection-transfer-preview-tile"
                                },
                                style: "{tile.style}",
                            }
                        }
                    })}

                    if let Some((selected_x, selected_y)) =
                        (!has_tile_selection_overlay).then_some(snapshot.selected_cell).flatten()
                    {
                        div {
                            key: "selected-cell-{selected_x}-{selected_y}",
                            class: "cell-hitbox selected",
                            style: cell_style(map.tile_width, map.tile_height, selected_x, selected_y),
                        }
                    }
                }
            }
        }
    }
}

fn object_intersects_visible_bounds(
    object: &taled_core::MapObject,
    visible_bounds: VisibleCellBounds,
    tile_width: u32,
    tile_height: u32,
) -> bool {
    let left = f64::from(visible_bounds.min_x * tile_width);
    let top = f64::from(visible_bounds.min_y * tile_height);
    let right = f64::from(visible_bounds.max_x * tile_width);
    let bottom = f64::from(visible_bounds.max_y * tile_height);

    let object_left = f64::from(object.x);
    let object_top = match object.shape {
        ObjectShape::Rectangle => f64::from(object.y),
        ObjectShape::Point => f64::from(object.y) - 30.0,
    };
    let object_right = match object.shape {
        ObjectShape::Rectangle => object_left + f64::from(object.width.max(1.0)),
        ObjectShape::Point => object_left + 20.0,
    };
    let object_bottom = match object.shape {
        ObjectShape::Rectangle => object_top + f64::from(object.height.max(1.0)),
        ObjectShape::Point => f64::from(object.y),
    };

    object_right >= left && object_left <= right && object_bottom >= top && object_top <= bottom
}

#[derive(Clone, Copy)]
struct VisibleCellBounds {
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

fn visible_cell_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    let (host_width, host_height) = snapshot.canvas_host_size.unwrap_or((384.0, 500.0));
    let zoom = (f64::from(snapshot.zoom_percent) / 100.0).max(0.01);
    let tile_width = f64::from(map.tile_width.max(1));
    let tile_height = f64::from(map.tile_height.max(1));
    let margin_x = tile_width;
    let margin_y = tile_height;

    let world_left = (-f64::from(snapshot.pan_x) - margin_x * zoom) / zoom;
    let world_top = (-f64::from(snapshot.pan_y) - margin_y * zoom) / zoom;
    let world_right = (host_width - f64::from(snapshot.pan_x) + margin_x * zoom) / zoom;
    let world_bottom = (host_height - f64::from(snapshot.pan_y) + margin_y * zoom) / zoom;

    let min_x = (world_left / tile_width).floor().max(0.0) as u32;
    let min_y = (world_top / tile_height).floor().max(0.0) as u32;
    let max_x = (world_right / tile_width).ceil().max(0.0) as u32;
    let max_y = (world_bottom / tile_height).ceil().max(0.0) as u32;

    VisibleCellBounds {
        min_x: min_x.min(map.width),
        max_x: max_x.max(min_x + 1).min(map.width),
        min_y: min_y.min(map.height),
        max_y: max_y.max(min_y + 1).min(map.height),
    }
}

fn expanded_visible_cell_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    const CACHE_MARGIN_TILES: u32 = 8;

    let visible = visible_cell_bounds(snapshot, map);
    VisibleCellBounds {
        min_x: visible.min_x.saturating_sub(CACHE_MARGIN_TILES),
        max_x: visible
            .max_x
            .saturating_add(CACHE_MARGIN_TILES)
            .min(map.width),
        min_y: visible.min_y.saturating_sub(CACHE_MARGIN_TILES),
        max_y: visible
            .max_y
            .saturating_add(CACHE_MARGIN_TILES)
            .min(map.height),
    }
}

fn full_map_cell_bounds(map: &taled_core::Map) -> VisibleCellBounds {
    VisibleCellBounds {
        min_x: 0,
        max_x: map.width,
        min_y: 0,
        max_y: map.height,
    }
}

fn visible_object_count(map: &taled_core::Map) -> usize {
    map.layers
        .iter()
        .filter_map(|layer| layer.as_object())
        .filter(|layer| layer.visible)
        .map(|layer| layer.objects.len())
        .sum()
}

fn prefers_full_flat_object_cache(map: &taled_core::Map) -> bool {
    const MAX_FULL_CACHE_AXIS_PX: u32 = 4_096;
    const MAX_FULL_CACHE_OBJECTS: usize = 2_000;

    map.total_pixel_width() <= MAX_FULL_CACHE_AXIS_PX
        && map.total_pixel_height() <= MAX_FULL_CACHE_AXIS_PX
        && visible_object_count(map) <= MAX_FULL_CACHE_OBJECTS
}

fn flat_object_cache_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    if prefers_full_flat_object_cache(map) {
        full_map_cell_bounds(map)
    } else {
        expanded_visible_cell_bounds(snapshot, map)
    }
}

#[cfg(target_arch = "wasm32")]
fn perf_now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn perf_now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn create_svg_url(svg: &str) -> String {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(svg));
    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("image/svg+xml");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &options) else {
        return String::new();
    };
    web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn create_svg_url(svg: &str) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(svg);
    format!("data:image/svg+xml;base64,{encoded}")
}

#[cfg(target_arch = "wasm32")]
fn revoke_svg_url(url: &str) {
    if url.starts_with("blob:") {
        let _ = web_sys::Url::revoke_object_url(url);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn revoke_svg_url(_url: &str) {}

fn flat_tile_cache_style(
    cache_bounds: Option<(u32, u32, u32, u32)>,
    map: &taled_core::Map,
) -> Option<String> {
    let (min_x, max_x, min_y, max_y) = cache_bounds?;
    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * map.tile_width,
        min_y * map.tile_height,
        (max_x.saturating_sub(min_x)).max(1) * map.tile_width,
        (max_y.saturating_sub(min_y)).max(1) * map.tile_height,
    ))
}

pub(crate) fn revoke_cache_urls(state: &AppState) {
    if let Some(url) = state.flat_object_layers_data_url.as_ref() {
        revoke_svg_url(url);
    }
}

pub(crate) fn rebuild_render_caches(state: &mut AppState) {
    rebuild_flat_tile_layer_cache(state);
    rebuild_flat_object_layer_cache(state);
    if state.active_tile_layer_separated {
        rebuild_active_tile_layer_cache(state);
    } else {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        state.active_tile_layer_cache_dirty = false;
    }
}

/// Rebuild only viewport-dependent caches (object layers).
/// Tile layers use a full-map canvas and don't need redraws on pan/zoom.
pub(crate) fn rebuild_viewport_caches(state: &mut AppState) {
    rebuild_flat_object_layer_cache(state);
}

pub(crate) fn rebuild_flat_tile_layer_cache(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    };

    let started_at_ms = perf_now_ms();
    let document = session.document();
    let map = &document.map;

    let excluded_layer = state
        .active_tile_layer_separated
        .then(|| active_tile_layer_index(map, state.active_layer))
        .flatten();
    let Some(draw_data) = collect_tile_draw_data(map, &state.image_cache, excluded_layer) else {
        eval_js(&clear_tile_img_js("taled-fc"));
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    };
    let tile_count = draw_data.tile_data.len() / 5;
    let js = generate_tile_draw_js(
        "taled-fc",
        &draw_data,
        map.total_pixel_width(),
        map.total_pixel_height(),
        map.tile_width,
        map.tile_height,
    );
    eval_js(&js);

    state.flat_tile_layers_data_url = Some(String::new());
    state.flat_tile_layers_cell_bounds = Some((0, map.width, 0, map.height));
    log(format!(
        "perf: flat-cache rebuilt strategy=canvas-blob tiles={tile_count} js_bytes={} duration_ms={:.1}",
        js.len(),
        perf_now_ms() - started_at_ms,
    ));
}

pub(crate) fn rebuild_active_tile_layer_cache(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    };

    let document = session.document();
    let map = &document.map;
    let Some(layer_index) = active_tile_layer_index(map, state.active_layer) else {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    };
    let Some(tile_layer) = map.layer(layer_index).and_then(|layer| layer.as_tile()) else {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    };
    if !tile_layer.visible {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    }

    let started_at_ms = perf_now_ms();
    let Some(draw_data) = collect_single_layer_draw_data(map, &state.image_cache, tile_layer)
    else {
        eval_js(&clear_tile_img_js("taled-ac"));
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    };
    let tile_count = draw_data.tile_data.len() / 5;
    let js = generate_tile_draw_js(
        "taled-ac",
        &draw_data,
        map.total_pixel_width(),
        map.total_pixel_height(),
        map.tile_width,
        map.tile_height,
    );
    eval_js(&js);

    state.active_tile_layer_data_url = Some(String::new());
    state.active_tile_layer_cell_bounds = Some((0, map.width, 0, map.height));
    log(format!(
        "perf: active-layer-cache rebuilt strategy=canvas-blob layer={layer_index} tiles={tile_count} js_bytes={} duration_ms={:.1}",
        js.len(),
        perf_now_ms() - started_at_ms,
    ));
}

pub(crate) fn rebuild_flat_object_layer_cache(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.flat_object_layers_data_url = None;
        state.flat_object_layers_cell_bounds = None;
        return;
    };

    let document = session.document();
    let map = &document.map;
    let cache_bounds = flat_object_cache_bounds(state, map);
    let strategy = if cache_bounds.min_x == 0
        && cache_bounds.min_y == 0
        && cache_bounds.max_x == map.width
        && cache_bounds.max_y == map.height
    {
        "full-map"
    } else {
        "slice"
    };
    let started_at_ms = perf_now_ms();

    let Some(svg) = build_flat_object_layers_svg(map, cache_bounds) else {
        state.flat_object_layers_data_url = None;
        state.flat_object_layers_cell_bounds = None;
        return;
    };

    let svg_bytes = svg.len();
    if let Some(old_url) = state.flat_object_layers_data_url.as_ref() {
        revoke_svg_url(old_url);
    }
    let url = create_svg_url(&svg);
    state.flat_object_layers_data_url = Some(url);
    state.flat_object_layers_cell_bounds = Some((
        cache_bounds.min_x,
        cache_bounds.max_x,
        cache_bounds.min_y,
        cache_bounds.max_y,
    ));
    log(format!(
        "perf: object-cache rebuilt strategy={strategy} bounds=({}, {})..({}, {}) cache_bytes={} duration_ms={:.1}",
        cache_bounds.min_x,
        cache_bounds.min_y,
        cache_bounds.max_x,
        cache_bounds.max_y,
        svg_bytes,
        perf_now_ms() - started_at_ms,
    ));
}

fn active_tile_layer_index(map: &taled_core::Map, active_layer: usize) -> Option<usize> {
    map.layer(active_layer)
        .and_then(|layer| layer.as_tile())
        .map(|_| active_layer)
}

fn build_flat_object_layers_svg(
    map: &taled_core::Map,
    cache_bounds: VisibleCellBounds,
) -> Option<String> {
    let slice_width =
        (cache_bounds.max_x.saturating_sub(cache_bounds.min_x)).max(1) * map.tile_width;
    let slice_height =
        (cache_bounds.max_y.saturating_sub(cache_bounds.min_y)).max(1) * map.tile_height;
    let mut body = String::new();
    let offset_x = f64::from(cache_bounds.min_x * map.tile_width);
    let offset_y = f64::from(cache_bounds.min_y * map.tile_height);

    for layer in &map.layers {
        let Some(object_layer) = layer.as_object() else {
            continue;
        };
        if !object_layer.visible {
            continue;
        }

        for object in &object_layer.objects {
            if let Some(object_svg) = flat_object_svg(
                object,
                cache_bounds,
                map.tile_width,
                map.tile_height,
                offset_x,
                offset_y,
            ) {
                body.push_str(&object_svg);
            }
        }
    }

    if body.is_empty() {
        return None;
    }

    Some(format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            "<g shape-rendering=\"crispEdges\">{}</g>",
            "</svg>"
        ),
        slice_width, slice_height, slice_width, slice_height, body,
    ))
}

fn flat_object_svg(
    object: &taled_core::MapObject,
    visible_bounds: VisibleCellBounds,
    tile_width: u32,
    tile_height: u32,
    offset_x: f64,
    offset_y: f64,
) -> Option<String> {
    if !object_intersects_visible_bounds(object, visible_bounds, tile_width, tile_height) {
        return None;
    }

    match object.shape {
        ObjectShape::Rectangle => Some(format!(
            concat!(
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" ",
                "fill=\"rgba(128,128,128,0.168)\" stroke=\"#808080\" stroke-width=\"0.5\"/>"
            ),
            f64::from(object.x) - offset_x,
            f64::from(object.y) - offset_y,
            f64::from(object.width.max(1.0)),
            f64::from(object.height.max(1.0)),
        )),
        ObjectShape::Point => Some(format!(
            concat!(
                "<g transform=\"translate({:.2},{:.2})\">",
                "<path fill=\"rgba(128,128,128,0.168)\" fill-rule=\"evenodd\" stroke=\"#808080\" stroke-width=\"1\" ",
                "d=\"M10 1C5.03 1 1 5.03 1 10c0 5.06 3.68 8.43 9 18 5.32-9.57 9-12.94 9-18 0-4.97-4.03-9-9-9Zm0 4.75a4.25 4.25 0 1 1 0 8.5a4.25 4.25 0 0 1 0-8.5Z\"/>",
                "</g>"
            ),
            f64::from(object.x) - 10.0 - offset_x,
            f64::from(object.y) - 30.0 - offset_y,
        )),
    }
}

fn collect_tile_draw_data(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    excluded_layer_index: Option<usize>,
) -> Option<TileDrawData> {
    let mut tile_data = Vec::new();
    let mut ts_dims = BTreeMap::new();

    for (layer_index, layer) in map.layers.iter().enumerate() {
        if excluded_layer_index == Some(layer_index) {
            continue;
        }
        let Some(tile_layer) = layer.as_tile() else {
            continue;
        };
        if !tile_layer.visible {
            continue;
        }
        collect_layer_tile_data(map, image_cache, tile_layer, &mut tile_data, &mut ts_dims);
    }

    (!tile_data.is_empty()).then_some(TileDrawData { tile_data, ts_dims })
}

fn collect_single_layer_draw_data(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    tile_layer: &taled_core::TileLayer,
) -> Option<TileDrawData> {
    let mut tile_data = Vec::new();
    let mut ts_dims = BTreeMap::new();
    collect_layer_tile_data(map, image_cache, tile_layer, &mut tile_data, &mut ts_dims);
    (!tile_data.is_empty()).then_some(TileDrawData { tile_data, ts_dims })
}

fn collect_layer_tile_data(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    tile_layer: &taled_core::TileLayer,
    tile_data: &mut Vec<u32>,
    ts_dims: &mut BTreeMap<usize, (u32, u32)>,
) {
    for y in 0..tile_layer.height {
        for x in 0..tile_layer.width {
            let gid = match tile_layer.tile_at(x, y) {
                Some(g) if g != 0 => g,
                _ => continue,
            };
            let tile_ref = match map.tile_reference_for_gid(gid) {
                Some(r) => r,
                None => continue,
            };
            if !image_cache.contains_key(&tile_ref.tileset_index) {
                continue;
            }
            let ts = &tile_ref.tileset.tileset;
            let columns = ts.columns.max(1);
            let sx = (tile_ref.local_id % columns) * ts.tile_width;
            let sy = (tile_ref.local_id / columns) * ts.tile_height;
            let dx = x * map.tile_width;
            let dy = y * map.tile_height;
            ts_dims
                .entry(tile_ref.tileset_index)
                .or_insert((ts.tile_width, ts.tile_height));
            tile_data.extend_from_slice(&[tile_ref.tileset_index as u32, sx, sy, dx, dy]);
        }
    }
}

struct TileDrawData {
    tile_data: Vec<u32>,
    ts_dims: BTreeMap<usize, (u32, u32)>,
}

fn generate_tile_draw_js(
    img_id: &str,
    draw_data: &TileDrawData,
    map_width_px: u32,
    map_height_px: u32,
    map_tile_width: u32,
    map_tile_height: u32,
) -> String {
    let data_str: String = draw_data
        .tile_data
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let dims_str: String = draw_data
        .ts_dims
        .iter()
        .map(|(idx, (w, h))| format!("{idx}:[{w},{h}]"))
        .collect::<Vec<_>>()
        .join(",");

    // Render to an offscreen canvas, then convert to blob URL for <img> display.
    // This avoids Android WebView canvas content loss during CSS transform changes.
    format!(
        concat!(
            "requestAnimationFrame(function _td(){{",
            "var ts=window._taledTs;if(!ts)return;",
            "var sd={{{dims}}};",
            "for(var k in sd){{if(!ts[k]||!ts[k].complete){{setTimeout(_td,50);return}}}}",
            "var c=document.createElement('canvas');",
            "c.width={mw};c.height={mh};",
            "var x=c.getContext('2d');",
            "x.imageSmoothingEnabled=false;",
            "var d=[{data}];",
            "var dw={tw},dh={th};",
            "for(var i=0;i<d.length;i+=5){{",
            "var s=sd[d[i]];",
            "x.drawImage(ts[d[i]],d[i+1],d[i+2],s[0],s[1],d[i+3],d[i+4],dw,dh)",
            "}}",
            "c.toBlob(function(b){{",
            "var el=document.getElementById('{iid}');",
            "if(!el)return;",
            "var old=el.dataset.blobUrl;",
            "if(old)URL.revokeObjectURL(old);",
            "var u=URL.createObjectURL(b);",
            "el.dataset.blobUrl=u;",
            "el.src=u",
            "}})",
            "}})",
        ),
        dims = dims_str,
        mw = map_width_px,
        mh = map_height_px,
        data = data_str,
        tw = map_tile_width,
        th = map_tile_height,
        iid = img_id,
    )
}

fn clear_tile_img_js(id: &str) -> String {
    format!(
        concat!(
            "requestAnimationFrame(function(){{",
            "var el=document.getElementById('{iid}');",
            "if(!el)return;",
            "var old=el.dataset.blobUrl;",
            "if(old)URL.revokeObjectURL(old);",
            "el.removeAttribute('src');el.dataset.blobUrl=''",
            "}})",
        ),
        iid = id,
    )
}

pub(crate) fn preload_tileset_images(image_cache: &BTreeMap<usize, String>) {
    let mut js = String::from("window._taledTs={};");
    for (index, data_uri) in image_cache {
        js.push_str(&format!(
            "window._taledTs[{index}]=new Image();window._taledTs[{index}].src=\"{data_uri}\";",
        ));
    }
    eval_js(&js);
}

fn collect_visible_tile_styles(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Vec<(u32, u32, String)> {
    let visible_bounds = visible_cell_bounds(snapshot, &document.map);
    document
        .map
        .layer(snapshot.active_layer)
        .and_then(|layer| layer.as_tile())
        .filter(|layer| layer.visible)
        .map(|tile_layer| {
            let max_y = visible_bounds.max_y.min(tile_layer.height);
            let max_x = visible_bounds.max_x.min(tile_layer.width);
            (visible_bounds.min_y..max_y)
                .flat_map(|y| (visible_bounds.min_x..max_x).map(move |x| (x, y)))
                .filter_map(|(x, y)| {
                    let gid = tile_layer.tile_at(x, y).filter(|gid| *gid != 0)?;
                    let style = sprite_style(document, &snapshot.image_cache, gid, x, y)?;
                    Some((x, y, style))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn refresh_flat_tile_layer_cache_if_needed(state: &mut AppState) {
    if state.session.is_none() {
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    }
    if state.flat_tile_layers_cell_bounds.is_none() {
        rebuild_flat_tile_layer_cache(state);
    }
}

pub(crate) fn refresh_flat_object_layer_cache_if_needed(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.flat_object_layers_data_url = None;
        state.flat_object_layers_cell_bounds = None;
        return;
    };

    let map = &session.document().map;
    let Some((cache_min_x, cache_max_x, cache_min_y, cache_max_y)) =
        state.flat_object_layers_cell_bounds
    else {
        rebuild_flat_object_layer_cache(state);
        return;
    };

    if cache_min_x == 0 && cache_min_y == 0 && cache_max_x == map.width && cache_max_y == map.height
    {
        return;
    }

    let visible = visible_cell_bounds(state, map);
    let fits_horizontally = visible.min_x >= cache_min_x && visible.max_x <= cache_max_x;
    let fits_vertically = visible.min_y >= cache_min_y && visible.max_y <= cache_max_y;
    if fits_horizontally && fits_vertically {
        return;
    }

    rebuild_flat_object_layer_cache(state);
}

pub(crate) fn refresh_active_tile_layer_cache_if_needed(state: &mut AppState) {
    if state.session.is_none() {
        state.active_tile_layer_data_url = None;
        state.active_tile_layer_cell_bounds = None;
        return;
    }
    if !state.active_tile_layer_separated {
        return;
    }
    if state.active_tile_layer_cell_bounds.is_none() {
        rebuild_active_tile_layer_cache(state);
    }
}

pub(crate) fn refresh_render_caches_if_needed(state: &mut AppState) {
    refresh_flat_tile_layer_cache_if_needed(state);
    refresh_flat_object_layer_cache_if_needed(state);
    refresh_active_tile_layer_cache_if_needed(state);
}

fn center_canvas_if_needed(state: &mut AppState, host_width: f64, host_height: f64) {
    if !state.pending_canvas_center || host_width <= 0.0 || host_height <= 0.0 {
        return;
    }

    let Some(session) = state.session.as_ref() else {
        return;
    };

    let map = &session.document().map;
    let zoom = f64::from(state.zoom_percent) / 100.0;
    let map_width = f64::from(map.total_pixel_width()) * zoom;
    let map_height = f64::from(map.total_pixel_height()) * zoom;

    state.pan_x = ((host_width - map_width) * 0.5).round() as i32;
    state.pan_y = ((host_height - map_height) * 0.5).round() as i32;
    state.pending_canvas_center = false;
    rebuild_render_caches(state);
    log(format!(
        "touch:center-map host=({host_width:.1},{host_height:.1}) map=({map_width:.1},{map_height:.1}) pan=({}, {}) zoom={}",
        state.pan_x, state.pan_y, state.zoom_percent,
    ));
}

fn sprite_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: u32,
    y: u32,
) -> Option<String> {
    let tile = document.map.tile_reference_for_gid(gid)?;
    let image = image_cache.get(&tile.tileset_index)?;
    let columns = tile.tileset.tileset.columns.max(1);
    let tile_width = tile.tileset.tileset.tile_width;
    let tile_height = tile.tileset.tileset.tile_height;
    let source_x = (tile.local_id % columns) * tile_width;
    let source_y = (tile.local_id / columns) * tile_height;

    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;background-image:url('{image}');background-position:-{}px -{}px;background-size:{}px {}px;image-rendering:pixelated;image-rendering:crisp-edges;",
        x * document.map.tile_width,
        y * document.map.tile_height,
        document.map.tile_width,
        document.map.tile_height,
        source_x,
        source_y,
        tile.tileset.tileset.image.width,
        tile.tileset.tileset.image.height,
    ))
}

fn cell_style(tile_width: u32, tile_height: u32, x: u32, y: u32) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        x * tile_width,
        y * tile_height,
        tile_width,
        tile_height
    )
}

fn signed_cell_style(tile_width: u32, tile_height: u32, x: i32, y: i32) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        x * tile_width as i32,
        y * tile_height as i32,
        tile_width,
        tile_height
    )
}

fn preview_tile_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: u32,
    y: u32,
) -> Option<String> {
    let mut style = sprite_style(document, image_cache, gid, x, y)?;
    style.push_str("opacity:0.46;filter:saturate(0.92);");
    Some(style)
}

fn preview_tile_style_signed(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: i32,
    y: i32,
) -> Option<String> {
    let tile = document.map.tile_reference_for_gid(gid)?;
    let image = image_cache.get(&tile.tileset_index)?;
    let columns = tile.tileset.tileset.columns.max(1);
    let tile_width = tile.tileset.tileset.tile_width;
    let tile_height = tile.tileset.tileset.tile_height;
    let source_x = (tile.local_id % columns) * tile_width;
    let source_y = (tile.local_id / columns) * tile_height;

    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;background-image:url('{image}');background-position:-{}px -{}px;background-size:{}px {}px;image-rendering:pixelated;image-rendering:crisp-edges;opacity:0.46;filter:saturate(0.92);",
        x * document.map.tile_width as i32,
        y * document.map.tile_height as i32,
        document.map.tile_width,
        document.map.tile_height,
        source_x,
        source_y,
        tile.tileset.tileset.image.width,
        tile.tileset.tileset.image.height,
    ))
}

fn build_shape_fill_preview(
    document: &EditorDocument,
    snapshot: &AppState,
    preview: crate::app_state::ShapeFillPreview,
) -> ShapeFillPreviewVisual {
    let (min_x, min_y, max_x, max_y) = preview_bounds(preview);
    let mut tiles = Vec::new();
    let preview_cells = shape_fill_cells(
        snapshot.shape_fill_mode,
        preview.start_cell.0,
        preview.start_cell.1,
        preview.end_cell.0,
        preview.end_cell.1,
    );

    for (x, y) in preview_cells {
        let style =
            preview_tile_style(document, &snapshot.image_cache, snapshot.selected_gid, x, y)
                .unwrap_or_else(|| {
                    cell_style(document.map.tile_width, document.map.tile_height, x, y)
                });
        tiles.push(ShapeFillPreviewTile {
            x: x as i32,
            y: y as i32,
            style,
            fallback: document
                .map
                .tile_reference_for_gid(snapshot.selected_gid)
                .is_none(),
        });
    }

    ShapeFillPreviewVisual {
        tiles,
        frame_style: preview_frame_style(
            document.map.tile_width,
            document.map.tile_height,
            min_x,
            min_y,
            max_x,
            max_y,
        ),
    }
}

fn preview_bounds(preview: crate::app_state::ShapeFillPreview) -> (u32, u32, u32, u32) {
    (
        preview.start_cell.0.min(preview.end_cell.0),
        preview.start_cell.1.min(preview.end_cell.1),
        preview.start_cell.0.max(preview.end_cell.0),
        preview.start_cell.1.max(preview.end_cell.1),
    )
}

fn preview_frame_style(
    tile_width: u32,
    tile_height: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * tile_width,
        min_y * tile_height,
        (max_x - min_x + 1) * tile_width,
        (max_y - min_y + 1) * tile_height,
    )
}

fn object_class(selected: Option<u32>, object_id: u32, shape: &ObjectShape) -> &'static str {
    match (selected == Some(object_id), shape) {
        (true, ObjectShape::Rectangle) => "object-overlay rectangle selected",
        (true, ObjectShape::Point) => "object-overlay point selected",
        (false, ObjectShape::Rectangle) => "object-overlay rectangle",
        (false, ObjectShape::Point) => "object-overlay point",
    }
}

struct ShapeFillPreviewVisual {
    tiles: Vec<ShapeFillPreviewTile>,
    frame_style: String,
}

struct TileSelectionTransferPreviewVisual {
    tiles: Vec<ShapeFillPreviewTile>,
}

struct ShapeFillPreviewTile {
    x: i32,
    y: i32,
    style: String,
    fallback: bool,
}

fn active_tile_selection_overlay(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Option<TileSelectionOverlayVisual> {
    if !is_tile_selection_tool(snapshot.tool) {
        return None;
    }
    let active_layer = document.map.layer(snapshot.active_layer)?;
    active_layer.as_tile()?;

    let closing_region = snapshot.tile_selection_closing;
    let (selection, selection_cells, preview, closing) =
        if let Some(preview_cells) = snapshot.tile_selection_preview_cells.clone() {
            let selection = selection_region_from_cells(&preview_cells)?;
            (selection, preview_cells, true, false)
        } else if let Some(selection) = snapshot.tile_selection_preview {
            (
                selection,
                selection_cells_from_region(selection),
                true,
                false,
            )
        } else if let (Some(selection), Some(selection_cells)) = (
            snapshot.tile_selection,
            snapshot.tile_selection_cells.clone(),
        ) {
            (selection, selection_cells, false, false)
        } else if snapshot
            .tile_selection_closing_started_at
            .is_some_and(|started_at| started_at.elapsed() <= TILE_SELECTION_FADE_DURATION)
        {
            let selection = closing_region?;
            (
                selection,
                snapshot
                    .tile_selection_closing_cells
                    .clone()
                    .unwrap_or_else(|| selection_cells_from_region(selection)),
                false,
                true,
            )
        } else {
            return None;
        };
    Some(build_tile_selection_overlay(
        document,
        selection,
        selection_cells,
        preview,
        closing,
        snapshot.tile_selection_transfer.is_some(),
    ))
}

fn build_tile_selection_overlay(
    document: &EditorDocument,
    selection: TileSelectionRegion,
    selection_cells: std::collections::BTreeSet<(i32, i32)>,
    preview: bool,
    closing: bool,
    transfer_active: bool,
) -> TileSelectionOverlayVisual {
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width_in_cells = max_x - min_x + 1;
    let height_in_cells = max_y - min_y + 1;
    let irregular = !selection_cells_are_rectangular(selection, &selection_cells);
    let show_handles =
        !irregular && !transfer_active && (width_in_cells > 1 || height_in_cells > 1);
    let show_irregular_handles = irregular && (width_in_cells > 1 || height_in_cells > 1);
    let region_style = signed_preview_frame_style(
        document.map.tile_width,
        document.map.tile_height,
        min_x,
        min_y,
        max_x,
        max_y,
    );

    TileSelectionOverlayVisual {
        preview,
        closing,
        irregular,
        region_style,
        cell_styles: if irregular {
            selection_cells
                .into_iter()
                .map(|(x, y)| {
                    signed_preview_frame_style(
                        document.map.tile_width,
                        document.map.tile_height,
                        x,
                        y,
                        x,
                        y,
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        show_handles,
        show_irregular_handles,
        handles: if show_handles || show_irregular_handles {
            vec![
                TileSelectionHandleVisual::new("top-left", "left:-11px;top:-11px;"),
                TileSelectionHandleVisual::new("top-right", "right:-11px;top:-11px;"),
                TileSelectionHandleVisual::new("bottom-left", "left:-11px;bottom:-11px;"),
                TileSelectionHandleVisual::new("bottom-right", "right:-11px;bottom:-11px;"),
            ]
        } else {
            Vec::new()
        },
    }
}

fn active_tile_selection_transfer_preview(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Option<TileSelectionTransferPreviewVisual> {
    let transfer = snapshot.tile_selection_transfer.as_ref()?;
    let selection = snapshot.tile_selection?;
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let mut tiles = Vec::new();

    for local_y in 0..transfer.height {
        for local_x in 0..transfer.width {
            let x = min_x + local_x as i32;
            let y = min_y + local_y as i32;
            let gid = transfer.tiles[(local_y * transfer.width + local_x) as usize];
            if gid == 0 {
                continue;
            }
            let style = preview_tile_style_signed(document, &snapshot.image_cache, gid, x, y)
                .unwrap_or_else(|| {
                    signed_cell_style(document.map.tile_width, document.map.tile_height, x, y)
                });
            tiles.push(ShapeFillPreviewTile {
                x,
                y,
                style,
                fallback: document.map.tile_reference_for_gid(gid).is_none(),
            });
        }
    }

    Some(TileSelectionTransferPreviewVisual { tiles })
}

struct TileSelectionOverlayVisual {
    preview: bool,
    closing: bool,
    irregular: bool,
    region_style: String,
    cell_styles: Vec<String>,
    show_handles: bool,
    show_irregular_handles: bool,
    handles: Vec<TileSelectionHandleVisual>,
}

struct TileSelectionHandleVisual {
    position: &'static str,
    style: &'static str,
}

impl TileSelectionHandleVisual {
    const fn new(position: &'static str, style: &'static str) -> Self {
        Self { position, style }
    }
}

fn dismiss_selection_from_outside_map_click(state: &mut AppState, x: f64, y: f64) {
    if !is_tile_selection_tool(state.tool) || state.tile_selection.is_none() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let active_layer = session.document().map.layer(state.active_layer);
    if active_layer.is_none_or(|layer| layer.as_tile().is_none()) {
        return;
    }
    if cell_from_surface(state, x, y).is_some() {
        return;
    }

    if state.tile_selection_transfer.is_some() {
        return;
    }
    clear_tile_selection_immediately(state);
    state.status = "Selection cleared.".to_string();
}

fn handle_canvas_click(state: &mut AppState, x: f64, y: f64) {
    dismiss_selection_from_outside_map_click(state, x, y);

    let Some(session) = state.session.as_ref() else {
        return;
    };
    let active_layer = session.document().map.layer(state.active_layer);
    if active_layer.is_none_or(|layer| layer.as_tile().is_none()) {
        return;
    }
    let Some((cell_x, cell_y)) = cell_from_surface(state, x, y) else {
        return;
    };
    apply_cell_tool(state, cell_x, cell_y);
}

fn signed_preview_frame_style(
    tile_width: u32,
    tile_height: u32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * tile_width as i32,
        min_y * tile_height as i32,
        ((max_x - min_x + 1) as u32) * tile_width,
        ((max_y - min_y + 1) as u32) * tile_height,
    )
}
