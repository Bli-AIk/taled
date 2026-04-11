use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::tile_palette::{crop_tile_texture, PaletteTile};
use super::widgets::{bottom_nav, editor_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let back = l10n::text(lang, "common-back");
    let done = l10n::text(lang, "common-done");
    page_header(
        ui,
        theme,
        &l10n::text(lang, "nav-tilesets"),
        Some((&back, MobileScreen::Editor)),
        Some((&done, MobileScreen::Editor)),
        state,
    );

    handle_sheet_pinch(state);

    ui.element()
        .id("tilesets-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((8, 14, 0, 14)).gap(6))
        .children(|ui| {
            let tile_info = state.session.as_ref().map(|session| {
                let map = &session.document().map;
                let ts = map.tilesets.first();
                let cols = ts.map_or(1, |t| t.tileset.columns.max(1));
                let count = ts.map_or(0, |t| t.tileset.tile_count);
                let first_gid = ts.map_or(1, |t| t.first_gid);
                let tw = ts.map_or(16, |t| t.tileset.tile_width);
                let th = ts.map_or(16, |t| t.tileset.tile_height);
                let name = ts.map_or(String::new(), |t| t.tileset.name.clone());
                (cols, count, first_gid, tw, th, name)
            });

            let Some((cols, count, first_gid, tw, th, ts_name)) = tile_info else {
                ui.text("Load a TMX sample to view tilesets.", |t| {
                    t.font_size(14).color(theme.muted_text)
                });
                return;
            };

            sprite_sheet_view(ui, state, cols, count, first_gid, tw, th);

            let sel_local = state.selected_gid.saturating_sub(first_gid);
            property_section(ui, state, theme, sel_local, &ts_name, tw, th, cols);
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Tilesets);
}

fn handle_sheet_pinch(state: &mut AppState) {
    let all_touches = touches();
    if all_touches.len() >= 2 {
        let t0 = all_touches[0].position;
        let t1 = all_touches[1].position;
        let dx = t1.x - t0.x;
        let dy = t1.y - t0.y;
        let dist = (dx * dx + dy * dy).sqrt() as f64;
        if dist > 10.0 {
            if let Some(prev) = state.sheet_pinch_dist {
                let ratio = dist / prev;
                state.sheet_zoom = (state.sheet_zoom * ratio as f32).clamp(0.3, 10.0);
            }
            state.sheet_pinch_dist = Some(dist);
        }
    } else {
        state.sheet_pinch_dist = None;
    }
}

fn sprite_sheet_view(
    ui: &mut Ui,
    state: &mut AppState,
    cols: u32,
    count: u32,
    first_gid: u32,
    tw: u32,
    th: u32,
) {
    let avail_w = screen_width() - 28.0;
    let sheet_w = cols as f32 * tw as f32;
    let fit_zoom = avail_w / sheet_w;
    let sheet_key = cols * 10000 + tw;
    if state.sheet_zoom <= 0.0 || state.sheet_zoom_key != sheet_key {
        state.sheet_zoom = fit_zoom;
        state.sheet_zoom_key = sheet_key;
    }

    let zoom = state.sheet_zoom;
    let cell_w = tw as f32 * zoom;
    let cell_h = th as f32 * zoom;
    let rows = count.div_ceil(cols);
    let is_pinching = touches().len() >= 2;

    ui.element()
        .id("sprite-sheet")
        .width(grow!())
        .height(grow!())
        .background_color(Color::from(0x101113_u32))
        .corner_radius(14.0)
        .layout(|l| l.direction(TopToBottom))
        .overflow(|o| o.scroll_x().scroll_y().clip())
        .children(|ui| {
            for row in 0..rows {
                sheet_row(ui, state, row, cols, count, first_gid, cell_w, cell_h, is_pinching);
            }
        });
}

fn sheet_row(
    ui: &mut Ui,
    state: &mut AppState,
    row: u32,
    cols: u32,
    count: u32,
    first_gid: u32,
    cell_w: f32,
    cell_h: f32,
    is_pinching: bool,
) {
    ui.element()
        .id(("sheet-row", row))
        .width(fit!())
        .height(fixed!(cell_h))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            for col in 0..cols {
                let idx = row * cols + col;
                if idx >= count {
                    break;
                }
                sheet_cell(ui, state, first_gid + idx, first_gid, cell_w, cell_h, is_pinching);
            }
        });
}

fn sheet_cell(
    ui: &mut Ui,
    state: &mut AppState,
    gid: u32,
    first_gid: u32,
    w: f32,
    h: f32,
    is_pinching: bool,
) {
    let is_selected = state.selected_gid == gid;
    let local_id = gid - first_gid;
    let tile = PaletteTile {
        gid,
        tileset_index: 0,
        local_id,
    };
    let tile_tex = crop_tile_texture(state, &tile);
    let sel_color = Color::u_rgba(255, 59, 48, 255);

    ui.element()
        .id(("cell", gid))
        .width(fixed!(w))
        .height(fixed!(h))
        .overflow(|o| o.clip())
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() && !is_pinching {
                state.selected_gid = gid;
            }
            if let Some(tex) = tile_tex {
                ui.element().width(grow!()).height(grow!()).image(tex).empty();
            }
            if is_selected {
                // Inset selection border overlay
                ui.element()
                    .id(("sel-border", gid))
                    .width(fixed!(w))
                    .height(fixed!(h))
                    .floating(|f| f.attach_parent().offset((0.0, 0.0)))
                    .border(|b| b.all(2).color(sel_color))
                    .empty();
            }
        });
}

fn property_section(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    local_id: u32,
    ts_name: &str,
    tw: u32,
    th: u32,
    cols: u32,
) {
    let expanded = state.property_panel_expanded;
    let arrow = if expanded { "▼" } else { "▲" };

    ui.element()
        .id("panel-toggle")
        .width(grow!())
        .height(fixed!(30.0))
        .background_color(theme.surface)
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY).direction(LeftToRight).gap(6))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.property_panel_expanded = !state.property_panel_expanded;
            }
            ui.text(arrow, |t| t.font_size(11).color(theme.muted_text));
            ui.text("Property Panel", |t| t.font_size(13).color(theme.muted_text));
        });

    if !expanded {
        return;
    }

    let x = (local_id % cols) * tw;
    let y = (local_id / cols) * th;

    ui.element()
        .id("prop-panel")
        .width(grow!())
        .height(fixed!(260.0))
        .background_color(theme.surface)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).padding((10, 12, 10, 12)).gap(8))
        .children(|ui| {
            general_column(ui, theme, local_id, ts_name, x, y, tw, th);
            ui.element()
                .width(fixed!(1.0))
                .height(grow!())
                .background_color(theme.border)
                .empty();
            custom_column(ui, theme);
        });
}

fn general_column(
    ui: &mut Ui,
    theme: &PlyTheme,
    local_id: u32,
    ts_name: &str,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) {
    ui.element()
        .id("general-col")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(5))
        .children(|ui| {
            ui.text("General", |t| t.font_size(14).color(theme.text));
            prop_row(ui, theme, "ID", &local_id.to_string());
            prop_row(ui, theme, "Class", &format!("{ts_name} Tileset"));
            prop_row(ui, theme, "X", &x.to_string());
            prop_row(ui, theme, "Y", &y.to_string());
            prop_row(ui, theme, "W", &w.to_string());
            prop_row(ui, theme, "H", &h.to_string());
            prop_row(ui, theme, "Probability", "1.0");
        });
}

fn custom_column(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .id("custom-col")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(5))
        .children(|ui| {
            ui.text("Custom Properties", |t| t.font_size(14).color(theme.text));
            prop_row(ui, theme, "IsWalkable", "true");
            prop_row(ui, theme, "Damage", "5");
            prop_row(ui, theme, "Type", "Water");
            ui.element()
                .id("add-prop-btn")
                .width(grow!())
                .height(fixed!(28.0))
                .corner_radius(8.0)
                .border(|b| b.all(1).color(theme.accent))
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    ui.text("+ Add Property", |t| {
                        t.font_size(13).color(theme.accent)
                    });
                });
        });
}

fn prop_row(ui: &mut Ui, theme: &PlyTheme, label: &str, value: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(24.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(4))
        .children(|ui| {
            ui.text(label, |t| t.font_size(11).color(theme.muted_text));
            ui.element()
                .width(grow!())
                .height(fixed!(20.0))
                .background_color(theme.border)
                .corner_radius(5.0)
                .layout(|l| l.padding((0, 6, 0, 6)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(value, |t| t.font_size(11).color(theme.text));
                });
        });
}
