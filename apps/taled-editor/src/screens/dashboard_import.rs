use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;
use crate::workspace;

/// Import action submenu popup (rendered as a floating overlay).
pub(crate) fn import_menu_popup(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if !state.show_import_menu {
        return;
    }

    let lang = state.resolved_language();
    let sw = screen_width();
    let sh = screen_height();

    // Backdrop
    ui.element()
        .id("import-menu-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 120))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_import_menu = false;
            }
        });

    // Menu card, positioned below the action buttons area
    let popup_w: f32 = 280.0;
    let popup_x = (sw - popup_w) / 2.0;
    ui.element()
        .id("import-menu-popup")
        .width(fixed!(popup_w))
        .height(fit!())
        .background_color(theme.surface_elevated)
        .corner_radius(16.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((popup_x, 140.0)))
        .layout(|l| l.direction(TopToBottom).padding((8, 12, 8, 12)))
        .children(|ui| {
            import_workspace_option(ui, state, theme, lang);

            // Separator
            ui.element()
                .width(grow!())
                .height(fixed!(1.0))
                .background_color(theme.border)
                .empty();

            import_tmx_option(ui, state, theme, lang);
        });
}

fn import_workspace_option(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    lang: crate::l10n::SupportedLanguage,
) {
    let label = l10n::text(lang, "import-menu-workspace");
    ui.element()
        .id("import-opt-workspace")
        .width(grow!())
        .height(fixed!(48.0))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((14, 0, 14, 0))
                .gap(10)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_import_menu = false;
                do_import_workspaces(state, lang);
            }
            let icon = state.icon_cache.get(IconId::Import);
            ui.element()
                .width(fixed!(18.0))
                .height(fixed!(18.0))
                .background_color(theme.text)
                .image(icon)
                .empty();
            ui.text(&label, |t| t.font_size(16).color(theme.text));
        });
}

fn import_tmx_option(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    lang: crate::l10n::SupportedLanguage,
) {
    let label = l10n::text(lang, "import-menu-tmx");
    ui.element()
        .id("import-opt-tmx")
        .width(grow!())
        .height(fixed!(48.0))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((14, 0, 14, 0))
                .gap(10)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_import_menu = false;
                do_import_tmx_files(state, lang);
            }
            let icon = state.icon_cache.get(IconId::NavProjects);
            ui.element()
                .width(fixed!(18.0))
                .height(fixed!(18.0))
                .background_color(theme.text)
                .image(icon)
                .empty();
            ui.text(&label, |t| t.font_size(16).color(theme.text));
        });
}

fn do_import_workspaces(state: &mut AppState, lang: crate::l10n::SupportedLanguage) {
    let (dirs, _) = workspace::scan_import_staging();
    if dirs.is_empty() {
        state.status = l10n::text(lang, "import-staging-empty");
        return;
    }
    let mut count = 0u32;
    for dir in &dirs {
        if workspace::import_directory_as_workspace(dir).is_some() {
            count += 1;
        }
    }
    state.workspace_list = workspace::list_workspaces()
        .into_iter()
        .map(|w| w.name)
        .collect();
    state.status = format!("{count} {}", l10n::text(lang, "import-workspace-done"));
}

fn do_import_tmx_files(state: &mut AppState, lang: crate::l10n::SupportedLanguage) {
    let (_, tmx_files) = workspace::scan_import_staging();
    if tmx_files.is_empty() {
        state.status = l10n::text(lang, "import-staging-empty");
        return;
    }
    let mut count = 0u32;
    for tmx in &tmx_files {
        if workspace::import_tmx_to_workspace(tmx, &state.active_workspace).is_some() {
            count += 1;
        }
    }
    state.status = format!("{count} {}", l10n::text(lang, "import-tmx-done"));
}
