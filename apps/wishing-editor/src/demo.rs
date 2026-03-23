#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::platform::{EMBEDDED_DEMO_MAP_PATH, log};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use wishing_core::EditorSession;

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_embedded_demo_session() -> wishing_core::Result<EditorSession> {
    log(format!(
        "boot: loading embedded demo map {EMBEDDED_DEMO_MAP_PATH}"
    ));
    EditorSession::load_embedded(
        EMBEDDED_DEMO_MAP_PATH,
        [
            (
                "maps/017-2.tmx",
                include_str!("../../../assets/samples/tmwa/maps/017-2.tmx")
                    .as_bytes()
                    .to_vec(),
            ),
            (
                "tilesets/collision.tsx",
                include_str!("../../../assets/samples/tmwa/tilesets/collision.tsx")
                    .as_bytes()
                    .to_vec(),
            ),
            (
                "tilesets/woodland_indoor.tsx",
                include_str!("../../../assets/samples/tmwa/tilesets/woodland_indoor.tsx")
                    .as_bytes()
                    .to_vec(),
            ),
            (
                "graphics/tiles/collision.png",
                include_bytes!("../../../assets/samples/tmwa/graphics/tiles/collision.png")
                    .to_vec(),
            ),
            (
                "graphics/tiles/woodland_indoor.png",
                include_bytes!("../../../assets/samples/tmwa/graphics/tiles/woodland_indoor.png")
                    .to_vec(),
            ),
        ],
    )
}

#[cfg(target_os = "android")]
pub(crate) fn load_embedded_demo_session() -> wishing_core::Result<EditorSession> {
    log(format!(
        "boot: loading embedded demo map {EMBEDDED_DEMO_MAP_PATH}"
    ));
    EditorSession::load_embedded(
        EMBEDDED_DEMO_MAP_PATH,
        [
            (
                "stage1-basic/map.tmx",
                include_str!("../../../assets/samples/stage1-basic/map.tmx")
                    .as_bytes()
                    .to_vec(),
            ),
            (
                "stage1-basic/terrain.tsx",
                include_str!("../../../assets/samples/stage1-basic/terrain.tsx")
                    .as_bytes()
                    .to_vec(),
            ),
            (
                "stage1-basic/terrain.png",
                include_bytes!("../../../assets/samples/stage1-basic/terrain.png").to_vec(),
            ),
        ],
    )
}
