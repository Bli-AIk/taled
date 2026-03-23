use js_sys::{Array, JSON};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{
    Blob, Document, ErrorEvent, Event, HtmlAnchorElement, HtmlElement, PromiseRejectionEvent, Url,
    window,
};

const BOOT_LOG_ID: &str = "wishing-room-boot-log";
const MAX_LOG_LINES: usize = 400;

thread_local! {
    static WEB_LOGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn install() {
    ensure_boot_overlay();
    log("boot: installing web diagnostics");

    console_error_panic_hook::set_once();
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = format!("panic: {panic_info}");
        log(message);
        previous_hook(panic_info);
    }));

    install_error_listener();
    install_unhandled_rejection_listener();
    record_environment();
}

pub fn mark_app_rendered() {
    log("boot: first render committed");
    if let Some(document) = document() {
        if let Some(element) = document.get_element_by_id(BOOT_LOG_ID) {
            let _ = element.set_attribute("style", "display:none;");
        }
    }
}

pub fn log(message: impl Into<String>) {
    let message = message.into();
    web_sys::console::log_1(&JsValue::from_str(&message));
    append_boot_line(&message);

    WEB_LOGS.with(|logs| {
        let mut logs = logs.borrow_mut();
        logs.push(message);
        if logs.len() > MAX_LOG_LINES {
            let extra = logs.len() - MAX_LOG_LINES;
            logs.drain(0..extra);
        }
    });
}

pub fn entries() -> Vec<String> {
    WEB_LOGS.with(|logs| logs.borrow().clone())
}

pub fn download_logs() -> Result<(), String> {
    let Some(document) = document() else {
        return Err("document unavailable".to_string());
    };
    let Some(body) = document.body() else {
        return Err("document body unavailable".to_string());
    };

    let lines = entries();
    let payload = if lines.is_empty() {
        "no logs captured".to_string()
    } else {
        lines.join("\n")
    };

    let content = Array::new();
    content.push(&JsValue::from_str(&payload));

    let blob = Blob::new_with_str_sequence(&content).map_err(js_error)?;
    let url = Url::create_object_url_with_blob(&blob).map_err(js_error)?;
    let anchor = document
        .create_element("a")
        .map_err(js_error)?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "failed to cast anchor element".to_string())?;

    anchor.set_href(&url);
    anchor.set_download("wishing-room-web.log");

    let _ = body.append_child(&anchor);
    anchor.click();
    let _ = body.remove_child(&anchor);
    let _ = Url::revoke_object_url(&url);

    Ok(())
}

fn install_error_listener() {
    let Some(win) = window() else {
        return;
    };

    let closure = Closure::<dyn FnMut(Event)>::wrap(Box::new(|event: Event| {
        if let Some(error) = event.dyn_ref::<ErrorEvent>() {
            log(format!("window error: {}", error.message()));
        } else {
            log(format!("window error event: {}", event.type_()));
        }
    }));

    let _ = win.add_event_listener_with_callback("error", closure.as_ref().unchecked_ref());
    closure.forget();
}

fn install_unhandled_rejection_listener() {
    let Some(win) = window() else {
        return;
    };

    let closure = Closure::<dyn FnMut(Event)>::wrap(Box::new(|event: Event| {
        if let Some(rejection) = event.dyn_ref::<PromiseRejectionEvent>() {
            log(format!(
                "unhandled rejection: {}",
                js_value_to_string(&rejection.reason())
            ));
        } else {
            log(format!("unhandled rejection event: {}", event.type_()));
        }
    }));

    let _ = win
        .add_event_listener_with_callback("unhandledrejection", closure.as_ref().unchecked_ref());
    closure.forget();
}

fn record_environment() {
    let Some(win) = window() else {
        return;
    };

    let width = win
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or_default();
    let height = win
        .inner_height()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or_default();
    let user_agent = win
        .navigator()
        .user_agent()
        .unwrap_or_else(|_| "unknown".to_string());

    log(format!(
        "env: viewport={width:.0}x{height:.0} dpr={:.2}",
        win.device_pixel_ratio(),
    ));
    log(format!("env: user-agent={user_agent}"));
}

fn ensure_boot_overlay() {
    let Some(document) = document() else {
        return;
    };
    if document.get_element_by_id(BOOT_LOG_ID).is_some() {
        return;
    }

    let Some(body) = document.body() else {
        return;
    };

    let element = match document.create_element("pre") {
        Ok(element) => element,
        Err(_) => return,
    };

    let overlay = match element.dyn_into::<HtmlElement>() {
        Ok(overlay) => overlay,
        Err(_) => return,
    };

    overlay.set_id(BOOT_LOG_ID);
    overlay.set_text_content(Some("Wishing Room web boot log\n"));
    let _ = overlay.set_attribute(
        "style",
        concat!(
            "position:fixed;inset:0;z-index:2147483647;margin:0;padding:12px;",
            "overflow:auto;background:#081019;color:#d8e6f3;font:12px/1.5 monospace;",
            "white-space:pre-wrap;"
        ),
    );
    let _ = body.append_child(&overlay);
}

fn append_boot_line(message: &str) {
    let Some(document) = document() else {
        return;
    };
    let Some(element) = document.get_element_by_id(BOOT_LOG_ID) else {
        return;
    };

    let existing = element.text_content().unwrap_or_default();
    let mut next = existing;
    next.push_str(message);
    next.push('\n');
    element.set_text_content(Some(&next));
}

fn document() -> Option<Document> {
    window().and_then(|win| win.document())
}

fn js_value_to_string(value: &JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            JSON::stringify(value)
                .ok()
                .and_then(|result| result.as_string())
        })
        .unwrap_or_else(|| format!("{value:?}"))
}

fn js_error(error: JsValue) -> String {
    js_value_to_string(&error)
}
