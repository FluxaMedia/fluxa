#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    FrameReady,
    VideoInfo,
    AudioInfo,
    Duration,
    LoadCompleted,
    UnloadCompleted,
    SeekDone,
    Playing,
    Paused,
    EndOfStream,
    NeedData,
    EnoughData,
    DroppedFrame,
    Error,
    Other(i32),
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub number: i64,
    pub text: String,
}

const ENDOFSTREAM: i32 = 0x1c;

impl Event {
    pub fn from_callback(raw_type: i32, number: i64, text: Option<&str>, webos_major: u32) -> Event {
        let ty = if webos_major < 5 && raw_type > ENDOFSTREAM { raw_type + 2 } else { raw_type };
        let kind = match ty {
            0x0 => EventKind::FrameReady,
            0x3 => EventKind::Duration,
            0x4 => EventKind::VideoInfo,
            0x7 => EventKind::AudioInfo,
            0x12 | 0x13 => EventKind::Error,
            0x16 => EventKind::LoadCompleted,
            0x17 => EventKind::UnloadCompleted,
            0x19 => EventKind::SeekDone,
            0x1a => EventKind::Playing,
            0x1b => EventKind::Paused,
            0x1c => EventKind::EndOfStream,
            0x26 => EventKind::NeedData,
            0x27 => EventKind::EnoughData,
            0x30 => EventKind::DroppedFrame,
            other => EventKind::Other(other),
        };
        Event { kind, number, text: text.unwrap_or_default().to_owned() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_firmware_shifts_late_events_by_two() {
        assert_eq!(Event::from_callback(0x24, 0, None, 4).kind, EventKind::NeedData);
        assert_eq!(Event::from_callback(0x26, 0, None, 5).kind, EventKind::NeedData);
    }

    #[test]
    fn early_events_are_not_shifted() {
        assert_eq!(Event::from_callback(0x16, 0, None, 4).kind, EventKind::LoadCompleted);
    }
}
