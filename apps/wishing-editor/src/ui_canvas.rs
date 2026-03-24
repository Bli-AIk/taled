use std::collections::BTreeMap;

use dioxus::prelude::*;
use wishing_core::{EditorDocument, ObjectShape};

use crate::{
    app_state::{AppState, Tool},
    edit_ops::apply_cell_tool,
    touch_ops::{
        handle_touch_pointer_cancel, handle_touch_pointer_down, handle_touch_pointer_move,
        handle_touch_pointer_up, should_ignore_synthetic_click,
    },
    ui_visuals::object_overlay_style,
};

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
    let zoom = snapshot.zoom_percent as f32 / 100.0;
    let canvas_style = format!(
        "width:{}px;height:{}px;transform:translate({}px, {}px) scale({zoom});",
        map.total_pixel_width(),
        map.total_pixel_height(),
        snapshot.pan_x,
        snapshot.pan_y
    );

    rsx! {
        div { class: "canvas-host",
            div {
                class: "canvas-stage",
                onmounted: move |event| {
                    let mut state = state;
                    async move {
                        if let Ok(rect) = event.get_client_rect().await {
                            state.write().canvas_stage_client_origin =
                                Some((rect.origin.x, rect.origin.y));
                        }
                    }
                },
                onpointerdown: move |event| handle_touch_pointer_down(&mut state.write(), event),
                onpointermove: move |event| handle_touch_pointer_move(&mut state.write(), event),
                onpointerup: move |event| handle_touch_pointer_up(&mut state.write(), event),
                onpointercancel: move |event| handle_touch_pointer_cancel(&mut state.write(), event),
                div {
                    class: "canvas",
                    style: canvas_style,
                    for (layer_index, layer) in map.layers.iter().enumerate() {
                        if let Some(tile_layer) = layer.as_tile() {
                            if tile_layer.visible {
                                for y in 0..tile_layer.height {
                                    for x in 0..tile_layer.width {
                                        if let Some(style) = tile_layer
                                            .tile_at(x, y)
                                            .filter(|gid| *gid != 0)
                                            .and_then(|gid| sprite_style(document, &snapshot.image_cache, gid, x, y))
                                        {
                                            div {
                                                key: "tile-{layer_index}-{x}-{y}",
                                                class: "tile-sprite",
                                                style: style,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for (layer_index, layer) in map.layers.iter().enumerate() {
                        if let Some(object_layer) = layer.as_object() {
                            if object_layer.visible {
                                for object in &object_layer.objects {
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
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for y in 0..map.height {
                        for x in 0..map.width {
                            div {
                                key: "cell-{x}-{y}",
                                class: if snapshot.selected_cell == Some((x, y)) { "cell-hitbox selected" } else { "cell-hitbox" },
                                style: cell_style(map.tile_width, map.tile_height, x, y),
                                onclick: move |_| {
                                    let mut state = state.write();
                                    if should_ignore_synthetic_click(&mut state) {
                                        return;
                                    }
                                    apply_cell_tool(&mut state, x, y);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
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
        "left:{}px;top:{}px;width:{}px;height:{}px;background-image:url('{image}');background-position:-{}px -{}px;background-size:{}px {}px;",
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

fn object_class(selected: Option<u32>, object_id: u32, shape: &ObjectShape) -> &'static str {
    match (selected == Some(object_id), shape) {
        (true, ObjectShape::Rectangle) => "object-overlay rectangle selected",
        (true, ObjectShape::Point) => "object-overlay point selected",
        (false, ObjectShape::Rectangle) => "object-overlay rectangle",
        (false, ObjectShape::Point) => "object-overlay point",
    }
}
