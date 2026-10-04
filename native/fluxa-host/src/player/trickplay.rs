use std::sync::mpsc::{self, Receiver};

use fluxa_core::player::trickplay::{self as format, BifFrame, Source};
use image::RgbaImage;

use super::*;

const SHEETS_KEPT: usize = 2;

pub(super) struct Trickplay {
    source: Source,
    wanted: Option<f64>,
    fetching: Option<Receiver<Option<Vec<u8>>>>,
    fetching_sheet: u64,
    decoding: Option<(u64, Receiver<Option<RgbaImage>>)>,
    sheets: Vec<(u64, RgbaImage)>,
    bif: Option<(Vec<u8>, Vec<BifFrame>)>,
    failed: bool,
}

pub(super) fn from_snapshot(snapshot: &Value) -> Option<Trickplay> {
    let index = snapshot
        .pointer("/player/currentStreamIndex")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let stream = snapshot
        .pointer("/player/currentStreams")?
        .as_array()?
        .get(index)?;
    format::source_from_stream(stream).map(Trickplay::new)
}

fn decode(bytes: &[u8]) -> Option<RgbaImage> {
    image::load_from_memory(bytes).ok().map(|image| image.to_rgba8())
}

fn decode_off_thread(bytes: Vec<u8>) -> Receiver<Option<RgbaImage>> {
    let (sender, receiver) = mpsc::channel();
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let _ = sender.send(decode(&bytes));
    });
    #[cfg(target_arch = "wasm32")]
    let _ = sender.send(decode(&bytes));
    receiver
}

fn thumbnail(time: f64, image: RgbaImage) -> Thumbnail {
    Thumbnail {
        time,
        size: [image.width() as usize, image.height() as usize],
        rgba: image.into_raw(),
    }
}

impl Trickplay {
    fn new(source: Source) -> Self {
        Self {
            source,
            wanted: None,
            fetching: None,
            fetching_sheet: 0,
            decoding: None,
            sheets: Vec::new(),
            bif: None,
            failed: false,
        }
    }

    pub(super) fn usable(&self) -> bool {
        !self.failed
    }

    pub(super) fn request(&mut self, time: f64) {
        self.wanted = Some(time);
    }

    fn drain(&mut self) {
        if let Some(result) = self.fetching.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.fetching = None;
            match (result, &self.source) {
                (None, _) => self.failed = true,
                (Some(bytes), Source::Bif { .. }) => match format::parse_bif(&bytes) {
                    Some(frames) => self.bif = Some((bytes, frames)),
                    None => self.failed = true,
                },
                (Some(bytes), Source::Sheets { .. }) => {
                    self.decoding = Some((self.fetching_sheet, decode_off_thread(bytes)));
                }
            }
        }
        if let Some((sheet, result)) = self
            .decoding
            .as_ref()
            .and_then(|(sheet, rx)| Some((*sheet, rx.try_recv().ok()?)))
        {
            self.decoding = None;
            match result {
                Some(image) => {
                    self.sheets.retain(|(index, _)| *index != sheet);
                    self.sheets.push((sheet, image));
                    if self.sheets.len() > SHEETS_KEPT {
                        self.sheets.remove(0);
                    }
                }
                None => self.failed = true,
            }
        }
    }

    pub(super) fn poll(&mut self, session: &fluxa_effects::SessionHandle) -> Option<Thumbnail> {
        if self.failed {
            return None;
        }
        self.drain();
        let time = self.wanted?;
        let millis = (time * 1000.0) as u64;
        match self.source.clone() {
            Source::Sheets {
                url,
                interval,
                tile_width,
                tile_height,
                count,
            } => {
                let tile = format::tile_at(millis, interval, tile_width, tile_height, count)?;
                let Some((_, sheet)) = self.sheets.iter().find(|(index, _)| *index == tile.sheet)
                else {
                    if self.fetching.is_none() && self.decoding.is_none() {
                        self.fetching_sheet = tile.sheet;
                        self.fetching = Some(session.fetch_bytes(format::sheet_url(&url, tile.sheet)));
                    }
                    return None;
                };
                let (width, height) = (
                    sheet.width() / tile_width as u32,
                    sheet.height() / tile_height as u32,
                );
                let tile = image::imageops::crop_imm(
                    sheet,
                    tile.column as u32 * width,
                    tile.row as u32 * height,
                    width,
                    height,
                )
                .to_image();
                self.wanted = None;
                Some(thumbnail(time, tile))
            }
            Source::Bif { url } => {
                let Some((bytes, frames)) = self.bif.as_ref() else {
                    if self.fetching.is_none() {
                        self.fetching = Some(session.fetch_bytes(url));
                    }
                    return None;
                };
                let frame = format::frame_at(frames, millis)?;
                let image = decode(&bytes[frame.start..frame.end])?;
                self.wanted = None;
                Some(thumbnail(time, image))
            }
        }
    }
}
