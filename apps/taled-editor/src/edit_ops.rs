use taled_core::{
    EditorError, EditorSession, Layer, MapObject, ObjectShape, Property, PropertyValue,
};

use crate::app_state::{AppState, Tool};

pub(crate) fn toggle_layer_visibility(state: &mut AppState, layer_index: usize) {
    apply_edit(state, |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.set_visible(!layer.visible());
        Ok(())
    });
}

pub(crate) fn toggle_layer_lock(state: &mut AppState, layer_index: usize) {
    apply_edit(state, |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.set_locked(!layer.locked());
        Ok(())
    });
}

pub(crate) fn rename_layer(state: &mut AppState, layer_index: usize, name: String) {
    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        *layer.name_mut() = name;
        Ok(())
    });
}

pub(crate) fn apply_cell_tool(state: &mut AppState, x: u32, y: u32) {
    state.selected_cell = Some((x, y));
    let layer_index = state.active_layer;
    match state.tool {
        Tool::Hand => {}
        Tool::Paint => {
            let gid = state.selected_gid;
            apply_edit(state, move |document| {
                let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
                    EditorError::Invalid(format!("unknown layer index {layer_index}"))
                })?;
                if layer.locked() {
                    return Err(EditorError::Invalid("layer is locked".to_string()));
                }
                let tile_layer = layer.as_tile_mut().ok_or_else(|| {
                    EditorError::Invalid("active layer is not a tile layer".to_string())
                })?;
                tile_layer.set_tile(x, y, gid)?;
                Ok(())
            });
        }
        Tool::Erase => {
            apply_edit(state, move |document| {
                let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
                    EditorError::Invalid(format!("unknown layer index {layer_index}"))
                })?;
                if layer.locked() {
                    return Err(EditorError::Invalid("layer is locked".to_string()));
                }
                let tile_layer = layer.as_tile_mut().ok_or_else(|| {
                    EditorError::Invalid("active layer is not a tile layer".to_string())
                })?;
                tile_layer.set_tile(x, y, 0)?;
                Ok(())
            });
        }
        Tool::Select => {}
        Tool::AddRectangle => create_object_at(state, ObjectShape::Rectangle, x, y),
        Tool::AddPoint => create_object_at(state, ObjectShape::Point, x, y),
    }
}

pub(crate) fn create_object(state: &mut AppState, shape: ObjectShape) {
    let cell = state.selected_cell.unwrap_or((0, 0));
    create_object_at(state, shape, cell.0, cell.1);
}

fn create_object_at(state: &mut AppState, shape: ObjectShape, x: u32, y: u32) {
    let layer_index = state.active_layer;
    let mut created = None;
    apply_edit(state, |document| {
        let id = document.map.next_object_id;
        document.map.next_object_id += 1;
        let tile_width = document.map.tile_width as f32;
        let tile_height = document.map.tile_height as f32;
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let object_layer = layer.as_object_mut().ok_or_else(|| {
            EditorError::Invalid("active layer is not an object layer".to_string())
        })?;
        object_layer.objects.push(MapObject {
            id,
            name: format!("Object {id}"),
            visible: true,
            x: x as f32 * tile_width,
            y: y as f32 * tile_height,
            width: if matches!(shape, ObjectShape::Rectangle) {
                tile_width
            } else {
                0.0
            },
            height: if matches!(shape, ObjectShape::Rectangle) {
                tile_height
            } else {
                0.0
            },
            shape: shape.clone(),
            properties: Vec::new(),
        });
        created = Some(id);
        Ok(())
    });
    state.selected_object = created;
}

pub(crate) fn nudge_selected_object(state: &mut AppState, dx: f32, dy: f32) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object_layer = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .ok_or_else(|| {
                EditorError::Invalid("active layer is not an object layer".to_string())
            })?;
        let object = object_layer
            .object_mut(object_id)
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.x += dx;
        object.y += dy;
        Ok(())
    });
}

pub(crate) fn delete_selected_object(state: &mut AppState) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object_layer = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .ok_or_else(|| {
                EditorError::Invalid("active layer is not an object layer".to_string())
            })?;
        object_layer
            .remove_object(object_id)
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        Ok(())
    });
    state.selected_object = None;
}

pub(crate) fn rename_selected_object(state: &mut AppState, name: String) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.name = name;
        Ok(())
    });
}

pub(crate) fn update_selected_object_geometry(
    state: &mut AppState,
    field: &'static str,
    raw: String,
) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let Ok(value) = raw.parse::<f32>() else {
        state.status = format!("Cannot parse '{raw}' as number.");
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        match field {
            "x" => object.x = value,
            "y" => object.y = value,
            "width" => object.width = value.max(0.0),
            "height" => object.height = value.max(0.0),
            _ => {}
        }
        Ok(())
    });
}

pub(crate) fn add_layer_property(state: &mut AppState) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.properties_mut().push(Property {
            name: "new_property".to_string(),
            value: PropertyValue::String(String::new()),
        });
        Ok(())
    });
}

pub(crate) fn remove_layer_property(state: &mut AppState, property_index: usize) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let properties = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut();
        if property_index < properties.len() {
            properties.remove(property_index);
        }
        Ok(())
    });
}

pub(crate) fn rename_layer_property(state: &mut AppState, property_index: usize, name: String) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut()
            .get_mut(property_index)
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.name = name;
        Ok(())
    });
}

pub(crate) fn update_layer_property_value(
    state: &mut AppState,
    property_index: usize,
    raw: String,
) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut()
            .get_mut(property_index)
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.value = property.value.parse_like(&raw)?;
        Ok(())
    });
}

pub(crate) fn add_object_property(state: &mut AppState) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.properties.push(Property {
            name: "new_property".to_string(),
            value: PropertyValue::String(String::new()),
        });
        Ok(())
    });
}

pub(crate) fn remove_object_property(state: &mut AppState, property_index: usize) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        if property_index < object.properties.len() {
            object.properties.remove(property_index);
        }
        Ok(())
    });
}

pub(crate) fn rename_object_property(state: &mut AppState, property_index: usize, name: String) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .and_then(|object| object.properties.get_mut(property_index))
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.name = name;
        Ok(())
    });
}

pub(crate) fn update_object_property_value(
    state: &mut AppState,
    property_index: usize,
    raw: String,
) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .and_then(|object| object.properties.get_mut(property_index))
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.value = property.value.parse_like(&raw)?;
        Ok(())
    });
}

pub(crate) fn selected_object_view(
    session: &EditorSession,
    selected_object: Option<u32>,
    layer_index: usize,
) -> Option<(&MapObject, usize)> {
    let object_id = selected_object?;
    let layer = session.document().map.layer(layer_index)?.as_object()?;
    let object = layer.object(object_id)?;
    Some((object, layer_index))
}

pub(crate) fn apply_edit<F>(state: &mut AppState, edit: F)
where
    F: FnOnce(&mut taled_core::EditorDocument) -> Result<(), EditorError>,
{
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };

    match session.edit(edit) {
        Ok(()) => state.status = "Edit applied.".to_string(),
        Err(error) => state.status = format!("Edit failed: {error}"),
    }
}
