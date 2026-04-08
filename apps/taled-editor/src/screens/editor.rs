use ply_engine::prelude::*;

use crate::app_state::{
    is_tile_selection_tool, AppState, MobileScreen, ShapeFillMode, TileSelectionMode, Tool,
};
use crate::canvas::render_canvas;
use crate::l10n;
use crate::session_ops::adjust_zoom;
use crate::theme::PlyTheme;

use super::tile_palette::{collect_palette_preview, render_tile_chip_grid};
use super::widgets::{bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    render_editor_header(ui, state, theme);
    render_tile_strip_shell(ui, state, theme);

    // Canvas fills remaining space between tile strip and toolbar
    let header_h = 56.0;
    let strip_h = 114.0;
    let toolbar_h = 68.0;
    let nav_h = 72.0;
    let canvas_h = screen_height() - header_h - strip_h - toolbar_h - nav_h;
    let canvas_w = screen_width();

    render_canvas(ui, state, theme, canvas_w, canvas_h);

    render_toolbar(ui, state, theme);

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Editor);
}

fn render_editor_header(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = state
        .session
        .as_ref()
        .map(|s| {
            s.document()
                .file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        })
        .unwrap_or_else(|| "Tile Map Editor".to_string());

    ui.element()
        .id("editor-header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((20, 16, 16, 16))
                .gap(6)
        })
        .children(|ui| {
            // Left: Back button (92px)
            let back = l10n::text(state.resolved_language(), "common-back");
            ui.element()
                .id("editor-back")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Dashboard);
                    }
                    ui.text(&back, |t| {
                        t.font_size(14)
                            .color(super::widgets::HEADER_ACTION_COLOR)
                    });
                });

            // Center: title
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(&title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right: Settings (92px)
            ui.element()
                .id("editor-settings")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Right, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Settings);
                    }
                    let settings = l10n::text(state.resolved_language(), "nav-settings");
                    ui.text(&settings, |t| {
                        t.font_size(14).color(theme.muted_text).alignment(Right)
                    });
                });
        });
}

/// Tile strip shell — 114px, sits between header and canvas.
/// Contains palette area (left) + side divider + tool panel (right).
fn render_tile_strip_shell(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let strip_bg = theme.surface_elevated;
    let divider_color = Color::rgba(1.0, 1.0, 1.0, 0.10);

    // Collect palette tiles (up to 24)
    let palette = collect_palette_preview(state, 24);

    ui.element()
        .id("tile-strip-shell")
        .width(grow!())
        .height(fixed!(114.0))
        .background_color(strip_bg)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            // Left: palette area with tile chip grid (2 rows, column-first flow)
            ui.element()
                .id("tile-palette")
                .width(grow!())
                .height(grow!())
                .overflow(|o| o.clip())
                .layout(|l| {
                    l.direction(TopToBottom)
                        .align(Left, Top)
                        .padding((10, 14, 10, 14))
                        .gap(6)
                })
                .children(|ui| {
                    render_tile_chip_grid(ui, state, theme, &palette);
                });

            // Vertical divider
            ui.element()
                .width(fixed!(1.0))
                .height(grow!())
                .layout(|l| l.padding((10, 0, 10, 0)))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(1.0))
                        .height(grow!())
                        .background_color(divider_color)
                        .corner_radius(0.5)
                        .empty();
                });

            // Right: tool side panel (62px)
            render_tool_side_panel(ui, state, theme);
        });
}

fn render_tool_side_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let selection_active = is_tile_selection_tool(state.tool);
    let shape_fill_active = state.tool == Tool::ShapeFill;

    ui.element()
        .id("tool-side-panel")
        .width(fixed!(62.0))
        .height(grow!())
        .layout(|l| {
            l.direction(TopToBottom)
                .align(CenterX, CenterY)
                .padding((8, 4, 8, 4))
                .gap(3)
        })
        .children(|ui| {
            if selection_active {
                render_selection_modes(ui, state, theme, lang);
            } else if shape_fill_active {
                render_shape_fill_modes(ui, state, theme, lang);
            } else {
                render_side_empty(ui, theme, lang);
            }
        });
}

fn render_side_empty(ui: &mut Ui, theme: &PlyTheme, lang: l10n::SupportedLanguage) {
    let empty_color = Color::u_rgb(0x6e, 0x6e, 0x73);
    let _ = theme;
    let line1 = l10n::text(lang, "tile-strip-side-empty-line-1");
    let line2 = l10n::text(lang, "tile-strip-side-empty-line-2");
    ui.text(&line1, |t| {
        t.font_size(9).color(empty_color).alignment(CenterX)
    });
    ui.text(&line2, |t| {
        t.font_size(9).color(empty_color).alignment(CenterX)
    });
}

fn render_mode_button(
    ui: &mut Ui,
    id: &'static str,
    label: &str,
    active: bool,
    glyph: &str,
) {
    let text_color = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        Color::u_rgb(0xd1, 0xd1, 0xd6)
    };
    let bg = if active {
        Color::rgba(142.0, 142.0, 147.0, 0.18)
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    };

    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(34.0))
        .background_color(bg)
        .corner_radius(9.0)
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(1))
        .on_press(move |_, _| {})
        .children(|ui| {
            ui.text(glyph, |t| t.font_size(11).color(text_color).alignment(CenterX));
            ui.text(label, |t| t.font_size(8).color(text_color).alignment(CenterX));
        });
}

fn render_selection_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(TileSelectionMode, &str, &str, &'static str); 4] = [
        (TileSelectionMode::Replace, "selection-mode-replace", "□", "sel-replace"),
        (TileSelectionMode::Add, "selection-mode-add", "□+", "sel-add"),
        (TileSelectionMode::Subtract, "selection-mode-subtract", "□−", "sel-sub"),
        (TileSelectionMode::Intersect, "selection-mode-intersect", "□□", "sel-inter"),
    ];
    for (mode, key, glyph, id) in &modes {
        let active = state.tile_selection_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        render_mode_button(ui, id, &label, active, glyph);
        if ui.just_released() {
            state.tile_selection_mode = mode_val;
        }
    }
}

fn render_shape_fill_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(ShapeFillMode, &str, &str, &'static str); 2] = [
        (ShapeFillMode::Rectangle, "shape-fill-mode-rectangle", "▭", "shp-rect"),
        (ShapeFillMode::Ellipse, "shape-fill-mode-ellipse", "◯", "shp-ellip"),
    ];
    for (mode, key, glyph, id) in &modes {
        let active = state.shape_fill_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        render_mode_button(ui, id, &label, active, glyph);
        if ui.just_released() {
            state.shape_fill_mode = mode_val;
        }
    }
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render_floating_controls(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // D-pad joystick (bottom-left, 92x92) — above canvas bottom
    ui.element()
        .id("dpad")
        .width(fixed!(92.0))
        .height(fixed!(92.0))
        .floating(|f| {
            f.anchor((Left, Bottom), (Left, Bottom))
                .offset((18.0, -18.0))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(46.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            // Simple D-pad cross
            ui.element()
                .width(fixed!(60.0))
                .height(fixed!(60.0))
                .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(4))
                .children(|ui| {
                    // Up
                    ui.element()
                        .id("dpad-up")
                        .width(fixed!(24.0))
                        .height(fixed!(14.0))
                        .layout(|l| l.align(Left, CenterY))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.camera_y -= 16.0;
                            }
                            ui.text("▲", |t| {
                                t.font_size(12).color(theme.muted_text).alignment(CenterX)
                            });
                        });

                    // Middle row (Left, Center, Right)
                    ui.element()
                        .width(fixed!(60.0))
                        .height(fixed!(18.0))
                        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
                        .children(|ui| {
                            ui.element()
                                .id("dpad-left")
                                .width(fixed!(14.0))
                                .height(fixed!(18.0))
                                .layout(|l| l.align(Left, CenterY))
                                .on_press(move |_, _| {})
                                .children(|ui| {
                                    if ui.just_released() {
                                        state.camera_x -= 16.0;
                                    }
                                    ui.text("◀", |t| {
                                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                                    });
                                });

                            ui.element()
                                .width(fixed!(16.0))
                                .height(fixed!(16.0))
                                .background_color(theme.surface)
                                .corner_radius(8.0)
                                .empty();

                            ui.element()
                                .id("dpad-right")
                                .width(fixed!(14.0))
                                .height(fixed!(18.0))
                                .layout(|l| l.align(Left, CenterY))
                                .on_press(move |_, _| {})
                                .children(|ui| {
                                    if ui.just_released() {
                                        state.camera_x += 16.0;
                                    }
                                    ui.text("▶", |t| {
                                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                                    });
                                });
                        });

                    // Down
                    ui.element()
                        .id("dpad-down")
                        .width(fixed!(24.0))
                        .height(fixed!(14.0))
                        .layout(|l| l.align(Left, CenterY))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.camera_y += 16.0;
                            }
                            ui.text("▼", |t| {
                                t.font_size(12).color(theme.muted_text).alignment(CenterX)
                            });
                        });
                });
        });

    // Zoom control (bottom-right, 118x42)
    ui.element()
        .id("zoom-float")
        .width(fixed!(118.0))
        .height(fixed!(42.0))
        .floating(|f| {
            f.anchor((Right, Bottom), (Right, Bottom))
                .offset((-18.0, -18.0))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(21.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            ui.element()
                .id("zoom-out")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        adjust_zoom(state, -25);
                    }
                    ui.text("−", |t| {
                        t.font_size(18).color(theme.text).alignment(CenterX)
                    });
                });

            ui.element()
                .width(fixed!(46.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    let zoom_text = format!("{}%", state.zoom_percent);
                    ui.text(&zoom_text, |t| {
                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                    });
                });

            ui.element()
                .id("zoom-in")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        adjust_zoom(state, 25);
                    }
                    ui.text("+", |t| {
                        t.font_size(18).color(theme.text).alignment(CenterX)
                    });
                });
        });
}

fn handle_tool_press(ui: &mut Ui, state: &mut AppState, tool: Tool) {
    if ui.just_released() {
        state.tool = tool;
    }
}

fn render_toolbar(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let tools: [(Tool, &str); 6] = [
        (Tool::Hand, "tool-hand"),
        (Tool::Paint, "tool-stamp"),
        (Tool::Fill, "tool-fill"),
        (Tool::Erase, "tool-eraser"),
        (Tool::Select, "tool-rect-select"),
        (Tool::ShapeFill, "tool-shape-fill"),
    ];

    // Toolbar bg matches reference surface color #1c1c1e
    let toolbar_bg = theme.surface;

    ui.element()
        .id("toolbar")
        .width(grow!())
        .height(fixed!(68.0))
        .background_color(toolbar_bg)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((6, 8, 6, 8))
                .gap(4)
        })
        .children(|ui| {
            for (i, (tool, label_key)) in tools.iter().enumerate() {
                let is_active = state.tool == *tool;
                let label = l10n::text(lang, label_key);
                let tool_val = *tool;
                let color = if is_active { theme.accent } else { theme.text };
                let bg = if is_active {
                    theme.accent_soft
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.0)
                };

                let icon_id = crate::icons::tool_icon_id(label_key);
                let icon_tex = state.icon_cache.get(icon_id);

                ui.element()
                    .id(("tool", i as u32))
                    .width(grow!())
                    .height(grow!())
                    .background_color(bg)
                    .corner_radius(10.0)
                    .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(4))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        handle_tool_press(ui, state, tool_val);
                        ui.element()
                            .width(fixed!(22.0))
                            .height(fixed!(22.0))
                            .background_color(color)
                            .image(icon_tex)
                            .empty();
                        ui.text(&label, |t| t.font_size(10).color(color));
                    });
            }
        });
}
