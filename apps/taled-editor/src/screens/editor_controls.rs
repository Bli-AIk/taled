use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::icons::IconId;
use crate::l10n;
use crate::session_ops::{adjust_zoom, apply_redo, apply_undo};
use crate::theme::PlyTheme;

pub(crate) fn render_history_buttons(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    safe_top: f32,
) {
    let session_can = state
        .session
        .as_ref()
        .map_or((false, false), |s| (s.can_undo(), s.can_redo()));
    let can_undo = !state.undo_action_order.is_empty() || session_can.0;
    let can_redo = !state.redo_action_order.is_empty() || session_can.1;

    let float_bg = Color::u_rgba(24, 24, 26, 245);
    let float_border = Color::u_rgba(255, 255, 255, 20);

    ui.element()
        .id("history-float")
        .floating(|f| {
            f.anchor((Left, Top), (Left, Top))
                .attach_root()
                .offset((6.0, 174.0 + safe_top))
                .z_index(12)
        })
        .layout(|l| l.direction(LeftToRight).gap(6))
        .children(|ui| {
            history_button(
                ui,
                state,
                theme,
                "undo",
                IconId::Undo,
                can_undo,
                float_bg,
                float_border,
                true,
            );
            history_button(
                ui,
                state,
                theme,
                "redo",
                IconId::Redo,
                can_redo,
                float_bg,
                float_border,
                false,
            );
        });
}

pub(crate) fn render_layer_panel(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    safe_top: f32,
) {
    let lang = state.resolved_language();
    let layer_name = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.layer(state.active_layer))
        .map_or_else(|| "\u{2014}".to_string(), |l| l.name().to_string());

    let float_bg = Color::u_rgba(24, 24, 26, 245);
    let float_border = Color::u_rgba(255, 255, 255, 20);
    let title_label = l10n::text(lang, "nav-layers");

    ui.element()
        .id("layer-float")
        .width(fixed!(158.0))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-6.0, 174.0 + safe_top))
                .z_index(12)
        })
        .background_color(float_bg)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(float_border))
        .layout(|l| {
            l.direction(LeftToRight)
                .padding((8, 10, 6, 10))
                .align(Left, CenterY)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.layers_panel_expanded = !state.layers_panel_expanded;
            }
            ui.element()
                .width(grow!())
                .layout(|l| l.direction(TopToBottom).gap(1))
                .children(|ui| {
                    ui.text(&title_label, |t| t.font_size(12).color(theme.text));
                    ui.text(&layer_name, |t| {
                        t.font_size(10).color(Color::u_rgba(255, 255, 255, 168))
                    });
                });
            ui.text("▽", |t| t.font_size(14).color(theme.muted_text));
        });
}

pub(crate) fn render_joystick_float(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
) {
    let base = 92.0_f32;
    let max_r = 28.0_f32;
    let knob = 36.0_f32;
    let joy_y = safe_top + 56.0 + 114.0 + canvas_h - base - 8.0;
    let cx = 8.0 + base / 2.0;
    let cy = joy_y + base / 2.0;

    ui.element()
        .id("joystick")
        .width(fixed!(base))
        .height(fixed!(base))
        .floating(|f| {
            f.anchor((Left, Top), (Left, Top))
                .attach_root()
                .offset((8.0, joy_y))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(base / 2.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            let (mx, my) = mouse_position();
            if ui.just_pressed() {
                state.joystick_active = true;
            }
            if state.joystick_active && ui.pressed() {
                let dx = mx - cx;
                let dy = my - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let (ox, oy) = if dist > max_r {
                    (dx * max_r / dist, dy * max_r / dist)
                } else {
                    (dx, dy)
                };
                state.joystick_offset = (ox, oy);
                let pan_speed = 3.0;
                state.pan_x -= ox * pan_speed / max_r;
                state.pan_y -= oy * pan_speed / max_r;
                state.canvas_dirty = true;
            }
            if !ui.pressed() {
                state.joystick_active = false;
                state.joystick_offset = (0.0, 0.0);
            }
            let (kx, ky) = state.joystick_offset;
            ui.element()
                .id("joy-knob")
                .width(fixed!(knob))
                .height(fixed!(knob))
                .floating(|f| {
                    f.anchor((CenterX, CenterY), (CenterX, CenterY))
                        .attach_parent()
                        .offset((kx, ky))
                })
                .background_color(theme.surface)
                .corner_radius(knob / 2.0)
                .border(|b| b.all(1).color(theme.border))
                .empty();
        });
}

pub(crate) fn render_zoom_slider(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
    extra_up: f32,
) {
    let track_w = 140.0_f32;
    let track_h = 42.0_f32;
    let handle = 28.0_f32;
    let max_offset = (track_w - handle) / 2.0 - 6.0;
    let zoom_y = safe_top + 56.0 + 114.0 + canvas_h - track_h - 8.0 - extra_up;
    let slider_cx = screen_width() - 8.0 - track_w / 2.0;

    ui.element()
        .id("zoom-slider")
        .width(fixed!(track_w))
        .height(fixed!(track_h))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-8.0, zoom_y))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(track_h / 2.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            let (mx, _) = mouse_position();
            if ui.just_pressed() {
                state.zoom_slider_active = true;
                state.zoom_accumulator = 0.0;
            }
            if state.zoom_slider_active && ui.pressed() {
                let dx = mx - slider_cx;
                let ox = dx.clamp(-max_offset, max_offset);
                state.zoom_slider_offset = ox;
                let zoom_speed = 1.5;
                state.zoom_accumulator += ox * zoom_speed / max_offset;
                if state.zoom_accumulator.abs() >= 1.0 {
                    let delta = state.zoom_accumulator as i32;
                    state.zoom_accumulator -= delta as f32;
                    adjust_zoom(state, delta);
                }
            }
            if !ui.pressed() {
                state.zoom_slider_active = false;
                state.zoom_slider_offset = 0.0;
                state.zoom_accumulator = 0.0;
            }
            // Zoom label
            let zoom_text = format!("{}%", state.zoom_percent);
            ui.text(&zoom_text, |t| {
                t.font_size(11).color(theme.muted_text).alignment(CenterX)
            });
            // Draggable handle
            let hx = state.zoom_slider_offset;
            ui.element()
                .id("zoom-handle")
                .width(fixed!(handle))
                .height(fixed!(handle))
                .floating(|f| {
                    f.anchor((CenterX, CenterY), (CenterX, CenterY))
                        .attach_parent()
                        .offset((hx, 0.0))
                })
                .background_color(theme.surface)
                .corner_radius(handle / 2.0)
                .border(|b| b.all(1).color(theme.border))
                .empty();
        });
}

// ── Private helpers ─────────────────────────────────────────────────

fn history_button(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    id: &'static str,
    icon_id: IconId,
    enabled: bool,
    bg: Color,
    border_color: Color,
    is_undo: bool,
) {
    let icon_color = if enabled {
        Color::u_rgba(255, 255, 255, 235)
    } else {
        Color::u_rgba(255, 255, 255, 87)
    };
    let btn_bg = if enabled {
        bg
    } else {
        Color::u_rgba(28, 28, 30, 148)
    };
    let icon_tex = state.icon_cache.get(icon_id);

    ui.element()
        .id(id)
        .width(fixed!(38.0))
        .height(fixed!(38.0))
        .background_color(btn_bg)
        .corner_radius(19.0)
        .border(|b| b.all(1).color(border_color))
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() && enabled {
                if is_undo {
                    apply_undo(state);
                } else {
                    apply_redo(state);
                }
            }
            ui.element()
                .width(fixed!(20.0))
                .height(fixed!(20.0))
                .background_color(icon_color)
                .image(icon_tex)
                .empty();
        });
}
