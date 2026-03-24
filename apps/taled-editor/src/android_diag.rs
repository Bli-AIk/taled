use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use jni::{
    JavaVM,
    objects::{JObject, JString, JValue},
};

const DEFAULT_LOG_PATH: &str =
    "/sdcard/Android/data/io.github.taled.editor/files/logs/taled-editor.log";

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_LOCK: Mutex<()> = Mutex::new(());

pub fn install() {
    let path = std::panic::catch_unwind(resolve_log_path)
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_PATH));
    let _ = LOG_PATH.set(path.clone());

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    append_line("boot: installing android diagnostics");
    append_line(format!("boot: log_path={}", path.display()));

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        append_line(format!("panic: {info}"));
        default_hook(info);
    }));
}

pub fn log(message: impl Into<String>) {
    append_line(message.into());
}

pub fn mark_app_rendered() {
    append_line("boot: first render committed");
}

pub fn log_path() -> String {
    LOG_PATH
        .get()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| DEFAULT_LOG_PATH.to_string())
}

fn append_line(message: impl Into<String>) {
    let path = LOG_PATH
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_PATH));
    let _guard = LOG_LOCK.lock().ok();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{timestamp}] {}", message.into());
    }
}

fn resolve_log_path() -> Option<PathBuf> {
    let context = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let android_context = unsafe { JObject::from_raw(context.context().cast()) };

    let external_dir = java_file_dir(&mut env, &android_context, true)
        .or_else(|| java_file_dir(&mut env, &android_context, false))?;

    Some(external_dir.join("logs").join("taled-editor.log"))
}

fn java_file_dir(
    env: &mut jni::JNIEnv<'_>,
    android_context: &JObject<'_>,
    prefer_external: bool,
) -> Option<PathBuf> {
    let file = if prefer_external {
        let null_object = JObject::null();
        env.call_method(
            android_context,
            "getExternalFilesDir",
            "(Ljava/lang/String;)Ljava/io/File;",
            &[JValue::Object(&null_object)],
        )
        .ok()?
        .l()
        .ok()?
    } else {
        env.call_method(android_context, "getFilesDir", "()Ljava/io/File;", &[])
            .ok()?
            .l()
            .ok()?
    };

    if file.is_null() {
        return None;
    }

    let absolute_path = env
        .call_method(file, "getAbsolutePath", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    let absolute_path = JString::from(absolute_path);
    let absolute_path: String = env.get_string(&absolute_path).ok()?.into();
    Some(PathBuf::from(absolute_path))
}
