use std::time::{Duration, Instant};

const KEYDOWN: u32 = 0x300;
const KEYUP: u32 = 0x301;
const SCANCODE_MASK: u32 = 1 << 30;
const DIRECTION_REPEAT: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RawKey {
    pub pressed: bool,
    pub repeat: bool,
    pub wcode: u32,
    pub sym: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Remote {
    Up,
    Down,
    Left,
    Right,
    Ok,
    Back,
    Backspace,
    Play,
    Pause,
    PlayPause,
    FastForward,
    Rewind,
    Stop,
    Exit,
    PageUp,
    PageDown,
    PointerHidden,
    Red,
    Green,
    Yellow,
    Blue,
}

fn word(raw: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_ne_bytes(raw.get(at..at + 4)?.try_into().ok()?))
}

pub fn decode(raw: &[u8]) -> Option<RawKey> {
    let kind = word(raw, 0)?;
    if kind != KEYDOWN && kind != KEYUP {
        return None;
    }
    let state = word(raw, 16)?;
    Some(RawKey {
        pressed: kind == KEYDOWN && state & 0xff == 1,
        repeat: state & 0x100 != 0,
        wcode: word(raw, 20)?,
        sym: word(raw, 24)?,
    })
}

pub fn map(key: &RawKey) -> Option<Remote> {
    match key.wcode {
        482 | 461 => return Some(Remote::Back),
        450 => return Some(Remote::Play),
        72 => return Some(Remote::Pause),
        261 => return Some(Remote::PlayPause),
        451 => return Some(Remote::FastForward),
        452 => return Some(Remote::Rewind),
        120 | 260 => return Some(Remote::Stop),
        505 => return Some(Remote::Exit),
        300 => return Some(Remote::PageUp),
        301 => return Some(Remote::PageDown),
        484 => return Some(Remote::PointerHidden),
        486 => return Some(Remote::Red),
        487 => return Some(Remote::Green),
        488 => return Some(Remote::Yellow),
        489 => return Some(Remote::Blue),
        _ => {}
    }
    Some(match key.sym {
        sym if sym == 82 | SCANCODE_MASK => Remote::Up,
        sym if sym == 81 | SCANCODE_MASK => Remote::Down,
        sym if sym == 80 | SCANCODE_MASK => Remote::Left,
        sym if sym == 79 | SCANCODE_MASK => Remote::Right,
        sym if sym == 88 | SCANCODE_MASK || sym == 77 | SCANCODE_MASK => Remote::Ok,
        sym if sym == 75 | SCANCODE_MASK => Remote::PageUp,
        sym if sym == 78 | SCANCODE_MASK => Remote::PageDown,
        13 => Remote::Ok,
        27 | 113 => Remote::Back,
        8 => Remote::Backspace,
        _ => return None,
    })
}

#[derive(Default)]
pub struct RepeatFilter {
    last: Option<Instant>,
}

impl RepeatFilter {
    pub fn accept(&mut self, remote: Remote, repeat: bool, now: Instant) -> bool {
        if !repeat {
            self.last = Some(now);
            return true;
        }
        let direction = matches!(remote, Remote::Up | Remote::Down | Remote::Left | Remote::Right);
        let due = self.last.is_none_or(|last| now.duration_since(last) >= DIRECTION_REPEAT);
        if direction && due {
            self.last = Some(now);
        }
        direction && due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: u32, state: u32, wcode: u32, sym: u32) -> Vec<u8> {
        let mut raw = vec![0u8; 56];
        raw[0..4].copy_from_slice(&kind.to_ne_bytes());
        raw[16..20].copy_from_slice(&state.to_ne_bytes());
        raw[20..24].copy_from_slice(&wcode.to_ne_bytes());
        raw[24..28].copy_from_slice(&sym.to_ne_bytes());
        raw
    }

    #[test]
    fn measured_dpad_and_ok_codes_map_through_sym() {
        let key = |wcode, sym| decode(&event(KEYDOWN, 1, wcode, sym)).unwrap();
        assert_eq!(map(&key(80, 80 | SCANCODE_MASK)), Some(Remote::Left));
        assert_eq!(map(&key(79, 79 | SCANCODE_MASK)), Some(Remote::Right));
        assert_eq!(map(&key(81, 81 | SCANCODE_MASK)), Some(Remote::Down));
        assert_eq!(map(&key(82, 82 | SCANCODE_MASK)), Some(Remote::Up));
        assert_eq!(map(&key(40, 13)), Some(Remote::Ok));
    }

    #[test]
    fn lg_private_codes_come_from_wcode_with_no_sym() {
        let key = |wcode| decode(&event(KEYDOWN, 1, wcode, 0)).unwrap();
        assert_eq!(map(&key(482)), Some(Remote::Back));
        assert_eq!(map(&key(450)), Some(Remote::Play));
        assert_eq!(map(&key(451)), Some(Remote::FastForward));
        assert_eq!(map(&key(505)), Some(Remote::Exit));
    }

    #[test]
    fn keyboard_scancode_in_wcode_is_not_a_remote_button() {
        let p_key = decode(&event(KEYDOWN, 1, 19, 'p' as u32)).unwrap();
        assert_eq!(map(&p_key), None);
    }

    #[test]
    fn release_and_repeat_flags_are_separate() {
        let up = decode(&event(KEYUP, 0, 482, 0)).unwrap();
        assert!(!up.pressed);
        let held = decode(&event(KEYDOWN, 0x101, 482, 0)).unwrap();
        assert!(held.pressed && held.repeat);
    }

    #[test]
    fn non_key_events_and_short_buffers_are_ignored() {
        assert!(decode(&event(0x400, 1, 0, 0)).is_none());
        assert!(decode(&[0; 8]).is_none());
    }

    #[test]
    fn colour_keys_use_the_measured_codes() {
        let key = |wcode| decode(&event(KEYDOWN, 1, wcode, 0)).unwrap();
        assert_eq!(map(&key(486)), Some(Remote::Red));
        assert_eq!(map(&key(489)), Some(Remote::Blue));
    }

    #[test]
    fn held_ok_never_repeats_and_held_arrows_are_throttled() {
        let start = Instant::now();
        let mut filter = RepeatFilter::default();
        assert!(filter.accept(Remote::Right, false, start));
        assert!(!filter.accept(Remote::Right, true, start + Duration::from_millis(25)));
        assert!(filter.accept(Remote::Right, true, start + Duration::from_millis(100)));
        assert!(!filter.accept(Remote::Ok, true, start + Duration::from_millis(500)));
    }
}
