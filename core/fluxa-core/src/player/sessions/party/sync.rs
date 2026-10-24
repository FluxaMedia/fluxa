use super::{ActiveParty, PartyMemberStatus, PartyPlayback};
use std::collections::HashSet;

pub const NUDGE_START_MS: i64 = 250;
pub const NUDGE_STOP_MS: i64 = 80;
pub const SEEK_THRESHOLD_MS: i64 = 1_500;
pub const PAUSED_TOLERANCE_MS: i64 = 500;
pub const NUDGE_FACTOR: f32 = 0.05;
pub const SEEK_NOTICE_MS: i64 = 2_000;
pub const REJOIN_GAP_MS: i64 = 2_000;
pub const BEHIND_STEP_MS: i64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PartyCorrection {
    InSync,
    Seek(i64),
    Nudge(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnSeek {
    Behind(i64),
    CatchUp,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartyAction {
    Paused,
    Resumed,
    Seeked,
    Speed,
}

pub fn expected_position_ms(playback: &PartyPlayback, now_ms: i64, duration_ms: i64) -> i64 {
    let position = if playback.paused {
        playback.position_ms
    } else {
        let elapsed = (now_ms - playback.at).max(0);
        playback.position_ms + (elapsed as f64 * playback.rate as f64) as i64
    };
    let position = position.max(0);
    if duration_ms > 0 {
        position.min(duration_ms)
    } else {
        position
    }
}

pub fn is_newer(candidate: &PartyPlayback, current: Option<&PartyPlayback>) -> bool {
    let Some(current) = current else {
        return true;
    };
    if candidate.seq != current.seq {
        return candidate.seq > current.seq;
    }
    candidate.member_id.as_deref().unwrap_or_default()
        > current.member_id.as_deref().unwrap_or_default()
}

pub fn correction(
    local_position_ms: i64,
    expected_position_ms: i64,
    playback: &PartyPlayback,
    nudging: bool,
) -> PartyCorrection {
    let drift = local_position_ms - expected_position_ms;
    let distance = drift.abs();
    if playback.paused {
        return if distance > PAUSED_TOLERANCE_MS {
            PartyCorrection::Seek(expected_position_ms)
        } else {
            PartyCorrection::InSync
        };
    }
    let threshold = if nudging {
        NUDGE_STOP_MS
    } else {
        NUDGE_START_MS
    };
    if distance > SEEK_THRESHOLD_MS {
        PartyCorrection::Seek(expected_position_ms)
    } else if distance > threshold {
        let factor = if drift > 0 {
            1.0 - NUDGE_FACTOR
        } else {
            1.0 + NUDGE_FACTOR
        };
        PartyCorrection::Nudge(playback.rate * factor)
    } else {
        PartyCorrection::InSync
    }
}

pub fn coordinator_id(party: &ActiveParty) -> Option<String> {
    let mut online: HashSet<&str> = party.presence.keys().map(String::as_str).collect();
    online.insert(party.member_id.as_str());
    if let Some(host) = party
        .host_member_id
        .as_deref()
        .filter(|host| online.contains(host) || party.host_only)
    {
        return Some(host.to_owned());
    }
    party
        .members
        .iter()
        .find(|member| online.contains(member.id.as_str()))
        .map(|member| member.id.clone())
        .or_else(|| Some(party.member_id.clone()))
}

pub fn action_of(previous: Option<&PartyPlayback>, next: &PartyPlayback) -> Option<PartyAction> {
    let previous = previous?;
    if previous.waiting || next.waiting || previous.media_key != next.media_key {
        return None;
    }
    if previous.paused != next.paused {
        return Some(if next.paused {
            PartyAction::Paused
        } else {
            PartyAction::Resumed
        });
    }
    if (previous.rate - next.rate).abs() > 0.01 {
        return Some(PartyAction::Speed);
    }
    if (next.position_ms - expected_position_ms(previous, next.at, 0)).abs() > SEEK_NOTICE_MS {
        return Some(PartyAction::Seeked);
    }
    None
}

pub fn own_seek(target_ms: i64, party_position_ms: i64, behind: bool) -> OwnSeek {
    if target_ms < party_position_ms - REJOIN_GAP_MS {
        OwnSeek::Behind(target_ms)
    } else if behind {
        OwnSeek::CatchUp
    } else {
        OwnSeek::Blocked
    }
}

pub fn should_rejoin(gap_ms: i64, playing: bool) -> bool {
    playing && gap_ms <= REJOIN_GAP_MS
}

pub fn reported_gap_ms(gap_ms: i64) -> i64 {
    ((gap_ms.max(0) + BEHIND_STEP_MS / 2) / BEHIND_STEP_MS * BEHIND_STEP_MS).max(BEHIND_STEP_MS)
}

pub fn waiting_member_ids(party: &ActiveParty, ignored: &HashSet<String>) -> Vec<String> {
    let key = party.media.key();
    party
        .presence
        .values()
        .filter(|presence| presence.media_key.as_deref() == Some(key.as_str()))
        .filter(|presence| {
            matches!(
                presence.status,
                PartyMemberStatus::Loading | PartyMemberStatus::Buffering
            )
        })
        .map(|presence| presence.member_id.clone())
        .filter(|id| !ignored.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn playback(seq: i64, member: &str, paused: bool, position_ms: i64, at: i64) -> PartyPlayback {
        PartyPlayback {
            seq,
            member_id: Some(member.to_owned()),
            paused,
            position_ms,
            rate: 1.0,
            at,
            media_key: None,
            waiting: false,
        }
    }

    #[test]
    fn simultaneous_actions_resolve_to_highest_member_id() {
        let a = playback(7, "a", true, 0, 0);
        let b = playback(7, "b", false, 0, 0);
        assert!(is_newer(&b, Some(&a)));
        assert!(!is_newer(&a, Some(&b)));
    }

    #[test]
    fn playing_position_advances_with_rate_and_clamps_to_duration() {
        let mut state = playback(1, "a", false, 10_000, 1_000);
        state.rate = 2.0;
        assert_eq!(expected_position_ms(&state, 3_000, 0), 14_000);
        assert_eq!(expected_position_ms(&state, 3_000, 12_000), 12_000);
    }

    #[test]
    fn small_drift_nudges_and_large_drift_seeks() {
        let state = playback(1, "a", false, 0, 0);
        assert_eq!(
            correction(1_000, 1_100, &state, false),
            PartyCorrection::InSync
        );
        assert!(
            matches!(correction(1_000, 1_400, &state, false), PartyCorrection::Nudge(r) if r > 1.0)
        );
        assert_eq!(
            correction(0, 5_000, &state, false),
            PartyCorrection::Seek(5_000)
        );
    }

    #[test]
    fn gap_reports_round_to_five_seconds() {
        assert_eq!(reported_gap_ms(1_000), 5_000);
        assert_eq!(reported_gap_ms(12_000), 10_000);
    }
}
