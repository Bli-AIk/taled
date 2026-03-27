#[allow(dead_code)]
pub(crate) const EMBEDDED_DEMO_MAP_PATH: &str =
    crate::embedded_samples::DEFAULT_EMBEDDED_SAMPLE_PATH;

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(target_os = "android")]
use jni::{JavaVM, objects::JObject};

#[cfg(target_os = "android")]
static ANDROID_BACK_PRESSES: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_arch = "wasm32")]
pub(crate) fn install() {
    crate::web_diag::install();
}

#[cfg(target_os = "android")]
pub(crate) fn install() {
    crate::android_diag::install();
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn install() {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn log(message: impl Into<String>) {
    crate::web_diag::log(message.into());
}

#[cfg(target_os = "android")]
pub(crate) fn log(message: impl Into<String>) {
    crate::android_diag::log(message.into());
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn log(_message: impl Into<String>) {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn mark_app_rendered() {
    crate::web_diag::mark_app_rendered();
}

#[cfg(target_os = "android")]
pub(crate) fn mark_app_rendered() {
    crate::android_diag::mark_app_rendered();
}

#[cfg(target_os = "android")]
pub(crate) fn log_path() -> Option<String> {
    Some(crate::android_diag::log_path())
}

#[cfg(target_os = "android")]
pub(crate) fn take_android_back_presses() -> usize {
    ANDROID_BACK_PRESSES.swap(0, Ordering::AcqRel)
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
#[cfg(not(target_os = "android"))]
pub(crate) fn take_android_back_presses() -> usize {
    0
}

#[cfg(target_os = "android")]
pub(crate) fn finish_app() {
    let context = ndk_context::android_context();
    let Ok(vm) = (unsafe { JavaVM::from_raw(context.vm().cast()) }) else {
        return;
    };
    let Ok(mut env) = vm.attach_current_thread() else {
        return;
    };
    let activity = unsafe { JObject::from_raw(context.context().cast()) };
    let _ = env.call_method(activity, "finish", "()V", &[]);
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
#[cfg(not(target_os = "android"))]
pub(crate) fn finish_app() {}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_dioxus_main_MainActivity_nativeOnBackPressed(
    _env: jni::JNIEnv<'_>,
    _activity: JObject<'_>,
) {
    ANDROID_BACK_PRESSES.fetch_add(1, Ordering::AcqRel);
}
