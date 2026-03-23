#[cfg(target_os = "android")]
mod android_diag;
mod app;
mod app_state;
mod demo;
mod edit_ops;
mod platform;
mod session_ops;
mod styles;
mod ui_canvas;
mod ui_inspector;
#[cfg(target_arch = "wasm32")]
mod web_diag;

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {
    platform::install();
    dioxus::launch(app::App);
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() {
    platform::install();
    dioxus::launch(app::App);
}
