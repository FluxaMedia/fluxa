use super::*;

#[derive(Clone, Debug, Default)]
pub struct VideoStatus {
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub has_frame: bool,
    pub error: Option<String>,
    pub chapters: Vec<(f64, String)>,
    pub buffering: Option<f32>,
    pub stalled: bool,
    pub buffered: f64,
    pub ended: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpeechEnergy {
    pub start: f64,
    pub frame_seconds: f64,
    pub energies: Vec<f64>,
}

pub struct Thumbnail {
    pub time: f64,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThumbnailPlan {
    pub enabled: bool,
    pub url: Option<String>,
    pub headers: Vec<(String, String)>,
    pub interval: f64,
    pub sheet: bool,
    pub keyframes: bool,
    pub hold: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ThumbnailProgress {
    pub done: usize,
    pub count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VideoTrack {
    pub id: String,
    pub subtitle: bool,
    pub language: Option<String>,
    pub title: Option<String>,
    pub selected: bool,
    pub external: bool,
    pub forced: bool,
    pub format: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TrackSelection {
    pub audio: Option<String>,
    pub subtitle: Option<String>,
    pub secondary_subtitle: Option<String>,
    pub subtitles_off: bool,
}

#[derive(Clone, Debug)]
pub enum VideoCommand {
    TogglePause,
    Seek(f64),
    SeekTo(f64),
    ToggleMute,
    SetVolume(f64),
    SetSpeed(f64),
    Shaders(String),
    Motion(String),
    Fit(String),
    SelectTracks(TrackSelection),
    Mpv(Vec<String>),
}

pub type DeviceOpener = Arc<
    dyn Fn(
            &wgpu::Adapter,
            &wgpu::DeviceDescriptor<'_>,
        ) -> Result<(wgpu::Device, wgpu::Queue), String>
        + Send
        + Sync,
>;

pub trait VideoBackend: Send {
    fn device_opener(&self) -> Option<DeviceOpener> {
        None
    }
    fn load(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str);
    fn configure(&mut self, _settings: &Value) {}
    fn load_preview(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str) {
        self.load(instance, device, url);
    }
    fn place_preview(&mut self, _rect: Option<[f32; 4]>) {}
    fn stop(&mut self);
    fn media_session(&mut self, _plan: &Value) {}
    fn command(&mut self, command: VideoCommand);
    fn render(&mut self, device: &wgpu::Device) -> Option<wgpu::TextureView>;
    fn status(&mut self) -> VideoStatus;
    fn tracks(&mut self) -> Vec<VideoTrack> {
        Vec::new()
    }
    fn analyze_audio(
        &mut self,
        _url: &str,
        _start: f64,
        _length: f64,
    ) -> Option<Receiver<Option<SpeechEnergy>>> {
        None
    }
    fn extract_subtitle(
        &mut self,
        _url: &str,
        _track: &str,
        _start: f64,
        _length: f64,
    ) -> Option<Receiver<Option<String>>> {
        None
    }
    fn thumbnails(&mut self, _plan: &ThumbnailPlan) {}
    fn thumbnail_progress(&mut self) -> Option<ThumbnailProgress> {
        None
    }
    fn release_start(&mut self) {}
    fn request_thumbnail(&mut self, _time: f64) {}
    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        None
    }
    fn passthrough(&self) -> bool {
        false
    }
    fn frame_driven(&self) -> bool {
        self.passthrough()
    }
    fn stats(&mut self) -> Option<PlaybackStats> {
        None
    }
    fn subtitle(&mut self) -> Option<String> {
        None
    }
}
