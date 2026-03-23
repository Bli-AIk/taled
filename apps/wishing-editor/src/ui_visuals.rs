use std::collections::BTreeMap;

use wishing_core::{EditorDocument, MapObject, ObjectShape};

use crate::app_state::PaletteTile;

const OBJECT_MAIN_HEX: &str = "#808080";
const OBJECT_FILL_RGBA: &str = "rgba(128,128,128,0.196)";
const OBJECT_SHADOW_RGBA: &str = "rgba(0,0,0,0.92)";
const OBJECT_SELECTED_RGBA: &str = "rgba(255,226,133,0.96)";
const PALETTE_PREVIEW_SIZE: f32 = 44.0;
const PALETTE_INSET: f32 = 4.0;
const POINT_MARKER_WIDTH: f32 = 20.0;
const POINT_MARKER_HEIGHT: f32 = 30.0;

pub(crate) fn palette_tile_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    tile: &PaletteTile,
) -> String {
    let Some(reference) = document.map.tile_reference_for_gid(tile.gid) else {
        return String::new();
    };
    let Some(image) = image_cache.get(&tile.tileset_index) else {
        return String::new();
    };

    let columns = reference.tileset.tileset.columns.max(1);
    let tile_width = reference.tileset.tileset.tile_width as f32;
    let tile_height = reference.tileset.tileset.tile_height as f32;
    let atlas_width = reference.tileset.tileset.image.width as f32;
    let atlas_height = reference.tileset.tileset.image.height as f32;
    let source_x = (tile.local_id % columns) as f32 * tile_width;
    let source_y = (tile.local_id / columns) as f32 * tile_height;

    let preview_box = PALETTE_PREVIEW_SIZE - PALETTE_INSET * 2.0;
    let scale = (preview_box / tile_width)
        .min(preview_box / tile_height)
        .max(1.0);
    let rendered_width = tile_width * scale;
    let rendered_height = tile_height * scale;
    let offset_x = (PALETTE_PREVIEW_SIZE - rendered_width) / 2.0 - source_x * scale;
    let offset_y = (PALETTE_PREVIEW_SIZE - rendered_height) / 2.0 - source_y * scale;

    format!(
        "background-image:url('{image}');background-position:{offset_x}px {offset_y}px;background-size:{}px {}px;",
        atlas_width * scale,
        atlas_height * scale,
    )
}

pub(crate) fn object_overlay_style(
    object: &MapObject,
    selectable: bool,
    selected: bool,
    zoom: f32,
) -> String {
    let pointer_events = if selectable { "auto" } else { "none" };
    match object.shape {
        ObjectShape::Rectangle => rectangle_overlay_style(object, pointer_events, selected),
        ObjectShape::Point => point_overlay_style(object, pointer_events, selected, zoom),
    }
}

pub(crate) fn object_icon_style(shape: &ObjectShape) -> String {
    let image = match shape {
        ObjectShape::Rectangle => rectangle_icon_data_uri(),
        ObjectShape::Point => point_marker_data_uri(),
    };

    format!(
        "background-image:{image};background-size:contain;background-repeat:no-repeat;background-position:center;"
    )
}

fn rectangle_overlay_style(object: &MapObject, pointer_events: &str, selected: bool) -> String {
    let selected_outline = if selected {
        "outline:2px solid rgba(255,226,133,0.96);outline-offset:1px;"
    } else {
        ""
    };

    format!(
        concat!(
            "left:{}px;top:{}px;width:{}px;height:{}px;pointer-events:{};",
            "border:1.5px solid {};background:{};box-shadow:1px 1px 0 {};",
            "{}"
        ),
        object.x,
        object.y,
        object.width.max(1.0),
        object.height.max(1.0),
        pointer_events,
        OBJECT_MAIN_HEX,
        OBJECT_FILL_RGBA,
        OBJECT_SHADOW_RGBA,
        selected_outline,
    )
}

fn point_overlay_style(
    object: &MapObject,
    pointer_events: &str,
    selected: bool,
    zoom: f32,
) -> String {
    let mut filters = vec![format!("drop-shadow(1px 1px 0 {OBJECT_SHADOW_RGBA})")];
    if selected {
        filters.push(format!("drop-shadow(0 0 4px {OBJECT_SELECTED_RGBA})"));
    }

    format!(
        concat!(
            "left:{}px;top:{}px;width:{}px;height:{}px;pointer-events:{};",
            "background-image:{};background-size:contain;background-repeat:no-repeat;background-position:center;",
            "transform-origin:center bottom;transform:scale({});filter:{};"
        ),
        object.x - POINT_MARKER_WIDTH / 2.0,
        object.y - POINT_MARKER_HEIGHT,
        POINT_MARKER_WIDTH,
        POINT_MARKER_HEIGHT,
        pointer_events,
        point_marker_data_uri(),
        (1.0 / zoom.max(0.01)),
        filters.join(" "),
    )
}

fn rectangle_icon_data_uri() -> String {
    svg_data_uri(
        r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 20'>
<rect x='3' y='3' width='14' height='14' fill='rgba(128,128,128,0.196)' stroke='#808080' stroke-width='1.5'/>
</svg>"#,
    )
}

fn point_marker_data_uri() -> String {
    svg_data_uri(
        r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 30'>
<path fill='rgba(128,128,128,0.196)' fill-rule='evenodd' stroke='#808080' stroke-width='1.5' d='M10 1C5.03 1 1 5.03 1 10c0 5.06 3.68 8.43 9 18 5.32-9.57 9-12.94 9-18 0-4.97-4.03-9-9-9Zm0 4.75a4.25 4.25 0 1 1 0 8.5a4.25 4.25 0 0 1 0-8.5Z'/>
</svg>"#,
    )
}

fn svg_data_uri(svg: &str) -> String {
    let encoded = svg
        .replace('%', "%25")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('"', "'")
        .replace('\n', "")
        .replace(' ', "%20");
    format!("url(\"data:image/svg+xml;utf8,{encoded}\")")
}
