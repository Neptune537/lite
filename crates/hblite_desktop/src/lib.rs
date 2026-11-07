pub mod club;
pub mod gui;
pub use gui::LemuriaNativeApp;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static LIVE_UPDATES: std::cell::RefCell<std::collections::VecDeque<nautilus_world::live_telemetry::LiveTelemetryUpdate>> = Default::default();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ingest_telemetry_update(json: &str, now_ms: f64) -> bool {
    if json.len() > 65_536 || !now_ms.is_finite() || now_ms < 0.0 { return false; }
    let Ok(update) = serde_json::from_str::<nautilus_world::live_telemetry::LiveTelemetryUpdate>(json) else { return false; };
    if !update.is_valid(now_ms as u64) { return false; }
    LIVE_UPDATES.with(|queue| {
        let mut queue = queue.borrow_mut();
        if queue.len() >= 128 { return false; }
        queue.push_back(update);
        true
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn telemetry_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn apply_live_telemetry(state: &mut nautilus_world::GodsEyeGlobalState) {
    let now = telemetry_now() as u64;
    LIVE_UPDATES.with(|queue| {
        for update in queue.borrow_mut().drain(..) {
            state.ingest_live_update(update, now);
        }
    });
    state.expire_live_tracks(now);
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Web runner handle for embedding the Dashboard into any web page or Cloudflare Pages canvas.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Mounts and starts the Dashboard (Reincarnated in a Different World) on the provided canvas.
    #[wasm_bindgen]
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(LemuriaNativeApp::default()))),
            )
            .await
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }
}


#[cfg(target_arch = "wasm32")]
pub fn play_radio_stream(url: &str, volume: f32) {
    if let Some(win) = web_sys::window() {
        let doc = match win.document() {
            Some(d) => d,
            None => return,
        };

        // Reuse or create #siren_radio_audio element
        let audio: web_sys::HtmlAudioElement = match doc.get_element_by_id("siren_radio_audio") {
            Some(elem) => match elem.dyn_into::<web_sys::HtmlAudioElement>() {
                Ok(a) => a,
                Err(_) => return,
            },
            None => {
                let elem = match doc.create_element("audio") {
                    Ok(e) => e,
                    Err(_) => return,
                };
                elem.set_id("siren_radio_audio");
                if let Some(body) = doc.body() {
                    let _ = body.append_child(&elem);
                }
                match elem.dyn_into::<web_sys::HtmlAudioElement>() {
                    Ok(a) => a,
                    Err(_) => return,
                }
            }
        };

        audio.set_cross_origin(Some("anonymous"));
        audio.set_volume(volume as f64);
        
        let current_src = audio.src();
        if !current_src.contains(url) {
            audio.set_src(url);
        }

        let _ = audio.play();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn pause_radio_stream() {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(elem) = doc.get_element_by_id("siren_radio_audio") {
                if let Ok(audio) = elem.dyn_into::<web_sys::HtmlAudioElement>() {
                    let _ = audio.pause();
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn set_radio_volume(volume: f32) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(elem) = doc.get_element_by_id("siren_radio_audio") {
                if let Ok(audio) = elem.dyn_into::<web_sys::HtmlAudioElement>() {
                    audio.set_volume(volume.clamp(0.0, 1.0) as f64);
                }
            }
        }
    }
}
