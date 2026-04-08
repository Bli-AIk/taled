use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    page_header(
        ui,
        theme,
        "Tile Property Editor",
        Some(("Back", MobileScreen::Editor)),
        Some(("Done", MobileScreen::Editor)),
        state,
    );

    ui.element()
        .id("tilesets-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((12, 14, 8, 14)).gap(10))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            // Extract tile info while session is borrowed
            let tile_info = state.session.as_ref().map(|session| {
                let map = &session.document().map;
                let selected_ref = map.tile_reference_for_gid(state.selected_gid);
                let selected_label = selected_ref
                    .as_ref()
                    .map(|r| format!("Selected Tile: ID {}", r.local_id))
                    .unwrap_or_else(|| "Selected Tile: ID 0".to_string());
                let tile_name = selected_ref
                    .as_ref()
                    .map(|r| format!("{} {}", r.tileset.tileset.name, r.local_id))
                    .unwrap_or_else(|| "terrain 0".to_string());
                let tile_type = selected_ref
                    .as_ref()
                    .map(|r| format!("{} Tileset", r.tileset.tileset.name))
                    .unwrap_or_else(|| "terrain Tileset".to_string());
                let cols = map.tilesets.first().map(|t| t.tileset.columns.max(1)).unwrap_or(1);
                let count = map.tilesets.first().map(|t| t.tileset.tile_count).unwrap_or(0);
                let first_gid = map.tilesets.first().map(|t| t.first_gid).unwrap_or(1);
                (selected_label, tile_name, tile_type, cols, count, first_gid)
            });

            let Some((selected_label, tile_name, tile_type, cols, count, first_gid)) = tile_info else {
                ui.text("Load an embedded TMX sample before opening tilesets.", |t| {
                    t.font_size(14).color(theme.muted_text)
                });
                return;
            };

            // Sprite Sheet View (section title: 17px, white text)
            ui.text("Sprite Sheet View", |t| t.font_size(17).color(theme.text));
            sprite_sheet_grid(ui, state, theme, cols, count, first_gid);

            // Selected tile info
            ui.text(&selected_label, |t| t.font_size(16).color(theme.text));

            // Name / Type property fields
            property_fields(ui, theme, &tile_name, &tile_type);

            // Custom Properties (section title with top gap)
            ui.element().width(grow!()).height(fixed!(10.0)).empty();
            ui.text("Custom Properties", |t| t.font_size(17).color(theme.text));
            custom_properties_card(ui, theme);

            // Footer note
            ui.text(
                "0 custom properties available. Collision editing stays out of scope for this pass.",
                |t| t.font_size(13).color(theme.muted_text),
            );
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Tilesets);
}

fn sprite_sheet_grid(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    cols: u32,
    count: u32,
    first_gid: u32,
) {
    let rows = count.div_ceil(cols);
    // Match Dioxus: 1px padding, 1px gaps, cells fill remaining space
    let total_gap = (cols - 1) as f32 + 2.0; // 1px gap * (cols-1) + 2px padding
    let cell_w = (384.0 - 28.0 - total_gap) / cols as f32;

    ui.element()
        .id("sprite-sheet")
        .width(grow!())
        .height(fit!())
        .background_color(theme.border_strong)
        .corner_radius(14.0)
        .layout(|l| l.direction(TopToBottom).padding((1, 1, 1, 1)).gap(1))
        .children(|ui| {
            for row in 0..rows {
                sheet_row(ui, state, theme, row, cols, count, first_gid, cell_w);
            }
        });
}

fn sheet_row(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    row: u32,
    cols: u32,
    count: u32,
    first_gid: u32,
    cell_w: f32,
) {
    ui.element()
        .id(("sheet-row", row))
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(LeftToRight).gap(1))
        .children(|ui| {
            for col in 0..cols {
                let idx = row * cols + col;
                if idx >= count {
                    break;
                }
                sheet_cell(ui, state, theme, first_gid + idx, cell_w);
            }
        });
}

fn sheet_cell(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, gid: u32, size: f32) {
    let is_selected = state.selected_gid == gid;
    let cell_bg = Color::from(0x101113_u32);
    ui.element()
        .id(("cell", gid))
        .width(fixed!(size))
        .height(fixed!(size))
        .background_color(cell_bg)
        .border(|b| {
            if is_selected {
                b.all(3).color(theme.accent)
            } else {
                b
            }
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.selected_gid = gid;
            }
        });
}

fn property_fields(ui: &mut Ui, theme: &PlyTheme, name: &str, tile_type: &str) {
    ui.element()
        .id("prop-fields")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom))
        .children(|ui| {
            field_row(ui, theme, "Name:", name);
            // separator
            ui.element()
                .width(grow!())
                .height(fixed!(1.0))
                .background_color(theme.border)
                .empty();
            field_row(ui, theme, "Type:", tile_type);
        });
}

fn field_row(ui: &mut Ui, theme: &PlyTheme, label: &str, value: &str) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((10, 14, 10, 14))
                .gap(12)
        })
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element()
                .width(grow!())
                .height(fixed!(40.0))
                .background_color(theme.border)
                .corner_radius(10.0)
                .layout(|l| l.padding((0, 14, 0, 14)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(value, |t| t.font_size(15).color(theme.text));
                });
        });
}

fn custom_properties_card(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .id("custom-props")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(TopToBottom)
                .align(CenterX, Top)
                .padding((14, 16, 14, 16))
                .gap(8)
        })
        .children(|ui| {
            ui.text(
                "No editable tile properties are available in the current Stage 1 data model.",
                |t| t.font_size(14).color(theme.muted_text).alignment(CenterX),
            );
            ui.text("+ Add Property", |t| {
                t.font_size(15).color(theme.accent).alignment(CenterX)
            });
        });
}
