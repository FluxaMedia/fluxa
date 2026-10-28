use crate::{MAX_DECODE_SIDE, resize_rgba_exact_with_filter};
use image::{
    AnimationDecoder, Frames, ImageDecoder, ImageFormat, RgbaImage,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
    imageops::FilterType,
};
use std::{collections::VecDeque, io::Cursor, sync::Arc, time::Duration};

const MIN_FRAME_DELAY: Duration = Duration::from_millis(20);
const MAX_FRAME_DELAY: Duration = Duration::from_secs(10);

pub struct StreamFrame {
    pub image: RgbaImage,
    pub duration: Duration,
}

pub trait FrameDecoder {
    fn next(&mut self) -> Result<Option<StreamFrame>, String>;
    fn rewind(&mut self) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub enum MediaSource {
    Bytes(Arc<[u8]>),
    Video(String),
}

pub fn is_video_url(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    [".mp4", ".webm", ".mkv", ".mov", ".m4v"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

pub struct FrameStream {
    decoder: Box<dyn FrameDecoder>,
    size: [u32; 2],
    target: [u32; 2],
    queue: VecDeque<StreamFrame>,
    live: bool,
}

impl FrameStream {
    pub fn open(bytes: Arc<[u8]>, target: [u32; 2]) -> Result<Option<Self>, String> {
        let decoder: Box<dyn FrameDecoder> = match image::guess_format(&bytes) {
            Ok(ImageFormat::Gif) => Box::new(CrateDecoder::open(bytes, Kind::Gif)?),
            Ok(ImageFormat::Png) => match CrateDecoder::open(bytes, Kind::Apng) {
                Ok(decoder) => Box::new(decoder),
                Err(_) => return Ok(None),
            },
            Ok(ImageFormat::WebP) => match open_webp(bytes)? {
                Some(decoder) => decoder,
                None => return Ok(None),
            },
            _ => return Ok(None),
        };
        let mut stream = Self {
            decoder,
            size: [0, 0],
            target,
            queue: VecDeque::new(),
            live: false,
        };
        let Some(first) = stream.decode()? else {
            return Ok(None);
        };
        let Some(second) = stream.decode()? else {
            return Ok(None);
        };
        stream.queue.push_back(first);
        stream.queue.push_back(second);
        Ok(Some(stream))
    }

    pub fn from_decoder(decoder: Box<dyn FrameDecoder>, target: [u32; 2]) -> Self {
        Self {
            decoder,
            size: [0, 0],
            target,
            queue: VecDeque::new(),
            live: true,
        }
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    pub fn poster(&self) -> &RgbaImage {
        &self.queue[0].image
    }

    pub fn next_frame(&mut self) -> Result<StreamFrame, String> {
        if let Some(frame) = self.queue.pop_front() {
            return Ok(frame);
        }
        if let Some(frame) = self.decode()? {
            return Ok(frame);
        }
        self.decoder.rewind()?;
        self.decode()?
            .ok_or_else(|| "animation has no frames after rewind".to_owned())
    }

    fn decode(&mut self) -> Result<Option<StreamFrame>, String> {
        let Some(frame) = self.decoder.next()? else {
            return Ok(None);
        };
        if self.live {
            return Ok(Some(frame));
        }
        let (width, height) = frame.image.dimensions();
        if self.size == [0, 0] {
            let scale = (self.target[0] as f32 / width.max(1) as f32)
                .min(self.target[1] as f32 / height.max(1) as f32)
                .min(1.0);
            self.size = [
                (width as f32 * scale).round().max(1.0) as u32,
                (height as f32 * scale).round().max(1.0) as u32,
            ];
        }
        Ok(Some(StreamFrame {
            image: resize_rgba_exact_with_filter(frame.image, self.size, FilterType::Triangle),
            duration: frame.duration,
        }))
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Gif,
    Webp,
    Apng,
}

struct CrateDecoder {
    bytes: Arc<[u8]>,
    kind: Kind,
    frames: Frames<'static>,
}

impl CrateDecoder {
    fn open(bytes: Arc<[u8]>, kind: Kind) -> Result<Self, String> {
        let frames = frames_for(&bytes, kind)?;
        Ok(Self {
            bytes,
            kind,
            frames,
        })
    }
}

fn limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_SIDE);
    limits.max_image_height = Some(MAX_DECODE_SIDE);
    limits.max_alloc = Some(128 * 1024 * 1024);
    limits
}

fn frames_for(bytes: &Arc<[u8]>, kind: Kind) -> Result<Frames<'static>, String> {
    let cursor = Cursor::new(Arc::clone(bytes));
    let error = |error: image::ImageError| error.to_string();
    match kind {
        Kind::Gif => {
            let mut decoder = GifDecoder::new(cursor).map_err(error)?;
            decoder.set_limits(limits()).map_err(error)?;
            Ok(decoder.into_frames())
        }
        Kind::Webp => {
            let mut decoder = WebPDecoder::new(cursor).map_err(error)?;
            if !decoder.has_animation() {
                return Err("not animated".to_owned());
            }
            decoder.set_limits(limits()).map_err(error)?;
            Ok(decoder.into_frames())
        }
        Kind::Apng => {
            let mut decoder = PngDecoder::new(cursor).map_err(error)?;
            if !decoder.is_apng().map_err(error)? {
                return Err("not animated".to_owned());
            }
            decoder.set_limits(limits()).map_err(error)?;
            Ok(decoder.apng().map_err(error)?.into_frames())
        }
    }
}

impl FrameDecoder for CrateDecoder {
    fn next(&mut self) -> Result<Option<StreamFrame>, String> {
        let Some(frame) = self.frames.next() else {
            return Ok(None);
        };
        let frame = frame.map_err(|error| error.to_string())?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let millis = if denominator == 0 {
            100
        } else {
            numerator / denominator
        };
        Ok(Some(StreamFrame {
            duration: Duration::from_millis(millis as u64).clamp(MIN_FRAME_DELAY, MAX_FRAME_DELAY),
            image: frame.into_buffer(),
        }))
    }

    fn rewind(&mut self) -> Result<(), String> {
        self.frames = frames_for(&self.bytes, self.kind)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn open_webp(bytes: Arc<[u8]>) -> Result<Option<Box<dyn FrameDecoder>>, String> {
    if let Some(decoder) = libwebp::LibWebp::open(Arc::clone(&bytes)) {
        return Ok(Some(Box::new(decoder)));
    }
    match CrateDecoder::open(bytes, Kind::Webp) {
        Ok(decoder) => Ok(Some(Box::new(decoder))),
        Err(error) if error == "not animated" => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(not(target_os = "linux"))]
fn open_webp(bytes: Arc<[u8]>) -> Result<Option<Box<dyn FrameDecoder>>, String> {
    match CrateDecoder::open(bytes, Kind::Webp) {
        Ok(decoder) => Ok(Some(Box::new(decoder))),
        Err(error) if error == "not animated" => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(target_os = "linux")]
mod libwebp {
    use super::*;
    use libloading::Library;
    use std::ffi::c_void;

    #[repr(C)]
    struct WebPData {
        bytes: *const u8,
        size: usize,
    }

    #[repr(C)]
    struct DecoderOptions {
        color_mode: i32,
        use_threads: i32,
        padding: [u32; 7],
    }

    #[repr(C)]
    struct AnimationInfo {
        canvas_width: u32,
        canvas_height: u32,
        loop_count: u32,
        background_color: u32,
        frame_count: u32,
        padding: [u32; 4],
    }

    type OptionsInit = unsafe extern "C" fn(*mut DecoderOptions, i32) -> i32;
    type DecoderNew =
        unsafe extern "C" fn(*const WebPData, *const DecoderOptions, i32) -> *mut c_void;
    type GetInfo = unsafe extern "C" fn(*const c_void, *mut AnimationInfo) -> i32;
    type HasMore = unsafe extern "C" fn(*const c_void) -> i32;
    type GetNext = unsafe extern "C" fn(*mut c_void, *mut *mut u8, *mut i32) -> i32;
    type Reset = unsafe extern "C" fn(*mut c_void);
    type DecoderDelete = unsafe extern "C" fn(*mut c_void);

    const DEMUX_ABI: i32 = 0x0107;

    pub struct LibWebp {
        decoder: *mut c_void,
        width: u32,
        height: u32,
        previous_timestamp: i32,
        has_more: HasMore,
        get_next: GetNext,
        reset: Reset,
        delete: DecoderDelete,
        _bytes: Arc<[u8]>,
        _library: Library,
    }

    impl LibWebp {
        pub fn open(bytes: Arc<[u8]>) -> Option<Self> {
            let library = unsafe {
                Library::new("libwebpdemux.so.2").or_else(|_| Library::new("libwebpdemux.so"))
            }
            .ok()?;
            unsafe {
                let options_init: OptionsInit =
                    *library.get(b"WebPAnimDecoderOptionsInitInternal\0").ok()?;
                let decoder_new: DecoderNew = *library.get(b"WebPAnimDecoderNewInternal\0").ok()?;
                let get_info: GetInfo = *library.get(b"WebPAnimDecoderGetInfo\0").ok()?;
                let has_more: HasMore = *library.get(b"WebPAnimDecoderHasMoreFrames\0").ok()?;
                let get_next: GetNext = *library.get(b"WebPAnimDecoderGetNext\0").ok()?;
                let reset: Reset = *library.get(b"WebPAnimDecoderReset\0").ok()?;
                let delete: DecoderDelete = *library.get(b"WebPAnimDecoderDelete\0").ok()?;

                let mut options = DecoderOptions {
                    color_mode: 1,
                    use_threads: 1,
                    padding: [0; 7],
                };
                if options_init(&mut options, DEMUX_ABI) == 0 {
                    return None;
                }
                let data = WebPData {
                    bytes: bytes.as_ptr(),
                    size: bytes.len(),
                };
                let decoder = decoder_new(&data, &options, DEMUX_ABI);
                if decoder.is_null() {
                    return None;
                }
                let mut info = AnimationInfo {
                    canvas_width: 0,
                    canvas_height: 0,
                    loop_count: 0,
                    background_color: 0,
                    frame_count: 0,
                    padding: [0; 4],
                };
                if get_info(decoder, &mut info) == 0
                    || info.frame_count < 2
                    || info.canvas_width == 0
                    || info.canvas_height == 0
                    || info.canvas_width > MAX_DECODE_SIDE
                    || info.canvas_height > MAX_DECODE_SIDE
                {
                    delete(decoder);
                    return None;
                }
                Some(Self {
                    decoder,
                    width: info.canvas_width,
                    height: info.canvas_height,
                    previous_timestamp: 0,
                    has_more,
                    get_next,
                    reset,
                    delete,
                    _bytes: bytes,
                    _library: library,
                })
            }
        }
    }

    impl FrameDecoder for LibWebp {
        fn next(&mut self) -> Result<Option<StreamFrame>, String> {
            unsafe {
                if (self.has_more)(self.decoder) == 0 {
                    return Ok(None);
                }
                let mut rgba = std::ptr::null_mut();
                let mut timestamp = 0_i32;
                if (self.get_next)(self.decoder, &mut rgba, &mut timestamp) == 0 || rgba.is_null() {
                    return Err("libwebp could not decode the next frame".to_owned());
                }
                let duration = Duration::from_millis(
                    timestamp.saturating_sub(self.previous_timestamp).max(0) as u64,
                )
                .clamp(MIN_FRAME_DELAY, MAX_FRAME_DELAY);
                self.previous_timestamp = timestamp;
                let length = self.width as usize * self.height as usize * 4;
                let source = std::slice::from_raw_parts(rgba, length);
                let image = RgbaImage::from_raw(self.width, self.height, source.to_vec())
                    .ok_or_else(|| "libwebp returned a short frame".to_owned())?;
                Ok(Some(StreamFrame { image, duration }))
            }
        }

        fn rewind(&mut self) -> Result<(), String> {
            unsafe { (self.reset)(self.decoder) };
            self.previous_timestamp = 0;
            Ok(())
        }
    }

    impl Drop for LibWebp {
        fn drop(&mut self) {
            unsafe { (self.delete)(self.decoder) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Delay, Frame, Rgba};

    fn gif(colors: &[u8], side: u32, delay_ms: u32) -> Arc<[u8]> {
        let frames = colors.iter().map(|&red| {
            Frame::from_parts(
                RgbaImage::from_pixel(side, side, Rgba([red, 0, 0, 255])),
                0,
                0,
                Delay::from_numer_denom_ms(delay_ms, 1),
            )
        });
        let mut bytes = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut bytes)
            .encode_frames(frames)
            .unwrap();
        bytes.into()
    }

    #[test]
    fn gif_frames_keep_their_delays_and_loop() {
        let mut stream = FrameStream::open(gif(&[255, 0], 8, 80), [64, 64])
            .unwrap()
            .unwrap();
        let first = stream.next_frame().unwrap();
        let second = stream.next_frame().unwrap();
        let wrapped = stream.next_frame().unwrap();
        assert_eq!(first.duration, Duration::from_millis(80));
        assert_ne!(first.image, second.image);
        assert_eq!(first.image, wrapped.image);
    }

    #[test]
    fn single_frame_gif_is_not_an_animation() {
        assert!(
            FrameStream::open(gif(&[10], 8, 80), [64, 64])
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn frames_are_scaled_to_the_target_without_upscaling() {
        let stream = FrameStream::open(gif(&[1, 2], 200, 40), [50, 50])
            .unwrap()
            .unwrap();
        assert_eq!(stream.size(), [50, 50]);
        let small = FrameStream::open(gif(&[1, 2], 20, 40), [50, 50])
            .unwrap()
            .unwrap();
        assert_eq!(small.size(), [20, 20]);
    }

    #[test]
    fn long_animation_keeps_every_frame() {
        let colors = (0..=255u8).collect::<Vec<_>>();
        let mut stream = FrameStream::open(gif(&colors, 8, 40), [8, 8])
            .unwrap()
            .unwrap();
        let reds = (0..256)
            .map(|_| stream.next_frame().unwrap().image.get_pixel(0, 0)[0])
            .collect::<Vec<_>>();
        assert_eq!(reds.len(), 256);
        assert!(reds.windows(2).all(|pair| pair[0] != pair[1]));
    }
}
