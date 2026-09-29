use wasm_bindgen::prelude::*;

/// Remuxes a complete in-memory MKV file into WebM (bitstream copy, no
/// re-encode) for MediaSource Extensions. For anything but small files,
/// prefer `IncrementalMkvRemuxer` below — this whole-buffer variant needs
/// the entire source downloaded first.
#[wasm_bindgen]
pub fn remux_mkv_to_webm(mkv_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    crate::demux::remux_mkv_to_webm(mkv_bytes).map_err(|e| JsValue::from_str(&e))
}

/// Streaming MKV -> WebM remuxer: feed it chunks as they arrive (e.g. from a
/// `fetch()` `ReadableStream`) via `push`, and append whatever bytes come
/// back to a `SourceBuffer` right away — playback can start before the
/// whole source has downloaded. Call `finish()` once at end-of-stream to
/// flush the final Cluster.
#[wasm_bindgen]
pub struct IncrementalMkvRemuxer {
    inner: crate::demux::IncrementalRemuxSession,
}

#[wasm_bindgen]
impl IncrementalMkvRemuxer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::demux::IncrementalRemuxSession::new(),
        }
    }

    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        self.inner.push(chunk)
    }

    pub fn finish(&mut self) -> Vec<u8> {
        self.inner.finish()
    }
}

impl Default for IncrementalMkvRemuxer {
    fn default() -> Self {
        Self::new()
    }
}
