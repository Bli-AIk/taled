use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header, section_label};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "properties-title");
    page_header(
        ui,
        theme,
        &title,
        Some(("Back", MobileScreen::Editor)),
        Some(("Done", MobileScreen::Editor)),
        state,
    );

    ui.element()
        .id("props-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(12).padding((14, 14, 0, 14)))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            // ── Session Actions ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "properties-session-title"),
            );
            session_actions_card(ui, state, theme);

            // ── View ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "properties-view-title"),
            );
            view_card(ui, state, theme);

            // ── Diagnostics ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "properties-diagnostics-title"),
            );
            diagnostics_card(ui, theme, &state.status.clone());

            // ── Export ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "properties-export-title"),
            );
            export_card(ui, theme);

            ui.element().width(grow!()).height(fixed!(20.0)).empty();
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Properties);
}

fn session_actions_card(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    settings_card(ui, theme, |ui, theme| {
        // Row: "Embedded Sample" | "Reload Default" (accent)
        action_row(
            ui,
            theme,
            "sa-reload",
            "Embedded Sample",
            &l10n::text(state.resolved_language(), "properties-action-reload"),
        );
        separator(ui, theme);
        // Row: "Save" | "Run" (accent)
        action_row(
            ui,
            theme,
            "sa-save",
            &l10n::text(state.resolved_language(), "properties-action-save"),
            &l10n::text(state.resolved_language(), "properties-action-run"),
        );
        separator(ui, theme);
        action_row(
            ui,
            theme,
            "sa-undo",
            &l10n::text(state.resolved_language(), "properties-action-undo"),
            &l10n::text(state.resolved_language(), "properties-action-run"),
        );
        separator(ui, theme);
        action_row(
            ui,
            theme,
            "sa-redo",
            &l10n::text(state.resolved_language(), "properties-action-redo"),
            &l10n::text(state.resolved_language(), "properties-action-run"),
        );
    });
}

fn view_card(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let zoom_pct = format!("{}%", state.zoom_percent);
    let theme_name = crate::theme::theme_choice_display_label(state);
    settings_card(ui, theme, |ui, theme| {
        // Zoom row with slider placeholder
        zoom_row(ui, theme, &zoom_pct);
        separator(ui, theme);
        action_row(
            ui,
            theme,
            "zm-minus",
            &format!(
                "{} -",
                l10n::text(state.resolved_language(), "properties-view-zoom")
            ),
            "Apply",
        );
        separator(ui, theme);
        action_row(
            ui,
            theme,
            "zm-plus",
            &format!(
                "{} +",
                l10n::text(state.resolved_language(), "properties-view-zoom")
            ),
            "Apply",
        );
        separator(ui, theme);
        info_row(
            ui,
            theme,
            &l10n::text(state.resolved_language(), "properties-view-theme"),
            &theme_name,
        );
    });
}

fn diagnostics_card(ui: &mut Ui, theme: &PlyTheme, status: &str) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(14))
        .children(|ui| {
            ui.text("Status", |t| t.font_size(15).color(theme.text));
            ui.text(status, |t| t.font_size(13).color(theme.muted_text));
        });
}

fn settings_card(ui: &mut Ui, theme: &PlyTheme, content: impl FnOnce(&mut Ui, &PlyTheme)) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.border)
        .corner_radius(14.0)
        .layout(|l| l.direction(TopToBottom).padding((0, 16, 0, 16)))
        .children(|ui| {
            content(ui, theme);
        });
}

fn action_row(ui: &mut Ui, theme: &PlyTheme, id: &'static str, label: &str, action_label: &str) {
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            ui.text(action_label, |t| t.font_size(15).color(theme.accent));
        });
}

fn zoom_row(ui: &mut Ui, theme: &PlyTheme, value: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(8))
        .children(|ui| {
            ui.text("Zoom", |t| t.font_size(15).color(theme.text));
            // Slider track (gray base with accent fill)
            ui.element()
                .width(grow!())
                .height(fixed!(6.0))
                .background_color(theme.border_strong)
                .corner_radius(3.0)
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(96.0))
                        .height(fixed!(6.0))
                        .background_color(theme.accent)
                        .corner_radius(3.0)
                        .empty();
                });
            ui.text(value, |t| t.font_size(15).color(theme.muted_text));
        });
}

fn info_row(ui: &mut Ui, theme: &PlyTheme, label: &str, value: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            ui.text(value, |t| t.font_size(15).color(theme.muted_text));
        });
}

fn export_card(ui: &mut Ui, theme: &PlyTheme) {
    settings_card(ui, theme, |ui, theme| {
        toggle_row(ui, theme, "exp-j", "JSON", true);
        separator(ui, theme);
        toggle_row(ui, theme, "exp-x", "XML", true);
        separator(ui, theme);
        toggle_row(ui, theme, "exp-p", "PNG", true);
    });
}

fn toggle_row(ui: &mut Ui, theme: &PlyTheme, id: &'static str, label: &str, enabled: bool) {
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            let bg = if enabled {
                theme.accent
            } else {
                theme.border_strong
            };
            ui.element()
                .width(fixed!(52.0))
                .height(fixed!(32.0))
                .background_color(bg)
                .corner_radius(16.0)
                .layout(|l| {
                    let ax = if enabled { Right } else { Left };
                    l.align(ax, CenterY).padding((0, 3, 0, 3))
                })
                .children(|ui| {
                    ui.element()
                        .width(fixed!(26.0))
                        .height(fixed!(26.0))
                        .background_color(theme.accent_text)
                        .corner_radius(13.0)
                        .empty();
                });
        });
}

fn separator(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .width(grow!())
        .height(fixed!(1.0))
        .layout(|l| l.padding((0, 16, 0, 16)))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(1.0))
                .background_color(theme.border)
                .empty();
        });
}
