use super::{
    ActiveParty, PartyAction, PartyClock, PartyCorrection, PartyError, PartyMedia, PartyMember,
    PartyMemberStatus, PartyPlayback, PartyPresence, PartySnapshot, action_of, coordinator_id,
    correction, expected_position_ms, is_newer, waiting_member_ids,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashSet;

pub const HEARTBEAT_INTERVAL_MS: i64 = 120_000;
pub const PERSIST_DELAY_MS: i64 = 1_500;
pub const STATUS_DELAY_MS: i64 = 250;
pub const RESUME_GRACE_MS: i64 = 3_000;
pub const WAIT_LIMIT_MS: i64 = 15_000;

#[derive(Clone, Copy, Debug)]
pub struct Time {
    pub local_ms: i64,
    pub wall_ms: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartyNotice {
    pub name: String,
    pub action: PartyAction,
    pub position_ms: i64,
    pub rate: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PartyEvent {
    MemberJoined(String),
    MemberLeft(String),
    Removed,
    Ended,
    MediaChanged(PartyMedia),
    ControlChanged(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PartyEffect {
    Send { event: &'static str, payload: Value },
    Track(Value),
    Persist { media: Option<PartyMedia>, playback: Option<PartyPlayback> },
    Heartbeat { announce: Option<String> },
    Rejoin,
    Chat(Value),
    Notice(PartyNotice),
    Event(PartyEvent),
    Finish(PartyEvent),
}

#[derive(Debug)]
pub struct PartySession {
    party: ActiveParty,
    playback: Option<PartyPlayback>,
    clock: PartyClock,
    local_presence: Option<PartyPresence>,
    status_due: Option<i64>,
    persist_due: Option<i64>,
    heartbeat_due: i64,
    ignored_waiting: HashSet<String>,
    wait_started: Option<i64>,
    resumed_at: Option<i64>,
    rejoining: bool,
}

fn decode<T: DeserializeOwned>(value: &Value) -> Option<T> {
    serde_json::from_value(value.clone()).ok()
}

fn text<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(Value::as_str)
}

fn encode<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

impl PartySession {
    pub fn new(
        party: ActiveParty,
        playback: Option<PartyPlayback>,
        mut clock: PartyClock,
        server_time: Option<(i64, i64, i64)>,
        at: i64,
    ) -> Self {
        if let Some((server_ms, sent_at, received_at)) = server_time {
            clock.reset();
            clock.add_sample(server_ms, sent_at, received_at);
        }
        let playback = playback.filter(|p| {
            p.media_key
                .as_deref()
                .is_none_or(|key| key == party.media.key())
        });
        Self {
            party,
            playback,
            clock,
            local_presence: None,
            status_due: None,
            persist_due: None,
            heartbeat_due: at + HEARTBEAT_INTERVAL_MS,
            ignored_waiting: HashSet::new(),
            wait_started: None,
            resumed_at: None,
            rejoining: false,
        }
    }

    pub fn party(&self) -> &ActiveParty {
        &self.party
    }

    pub fn playback(&self) -> Option<&PartyPlayback> {
        self.playback.as_ref()
    }

    pub fn clock_mut(&mut self) -> &mut PartyClock {
        &mut self.clock
    }

    pub fn now_ms(&self, t: Time) -> i64 {
        self.clock.now_ms(t.local_ms, t.wall_ms)
    }

    pub fn correction_for(
        &self,
        t: Time,
        local_position_ms: i64,
        duration_ms: i64,
        nudging: bool,
    ) -> Option<PartyCorrection> {
        let playback = self.playback.as_ref()?;
        let expected = expected_position_ms(playback, self.now_ms(t), duration_ms);
        Some(correction(local_position_ms, expected, playback, nudging))
    }

    fn own_presence(&self) -> PartyPresence {
        self.local_presence
            .clone()
            .unwrap_or_else(|| PartyPresence::browsing(&self.party.member_id))
    }

    fn send_state(&self, playback: &PartyPlayback) -> PartyEffect {
        PartyEffect::Send {
            event: "state",
            payload: encode(playback),
        }
    }

    pub fn on_connection(&mut self, connected: bool) -> Vec<PartyEffect> {
        if !connected {
            let own = self.party.member_id.clone();
            self.party.connected = false;
            self.party.presence.retain(|key, _| *key == own);
            return Vec::new();
        }
        self.party.connected = true;
        let presence = encode(&self.own_presence());
        let mut effects = vec![
            PartyEffect::Track(presence.clone()),
            PartyEffect::Send {
                event: "status",
                payload: presence,
            },
        ];
        if let Some(playback) = &self.playback {
            effects.push(self.send_state(playback));
        }
        effects.push(PartyEffect::Send {
            event: "state_request",
            payload: json!({"member_id": self.party.member_id}),
        });
        effects
    }

    pub fn on_message(&mut self, event: &str, payload: &Value, t: Time) -> Vec<PartyEffect> {
        match event {
            "state" => decode::<PartyPlayback>(payload)
                .map(|candidate| self.accept(candidate, t))
                .unwrap_or_default(),
            "state_request" => {
                if text(payload, "member_id") == Some(self.party.member_id.as_str()) {
                    return Vec::new();
                }
                let mut effects = vec![PartyEffect::Send {
                    event: "status",
                    payload: encode(&self.own_presence()),
                }];
                if let Some(playback) = &self.playback {
                    effects.push(self.send_state(playback));
                }
                effects
            }
            "status" => decode::<PartyPresence>(payload)
                .map(|status| self.apply_status(status))
                .unwrap_or_default(),
            "control" => {
                let Some(host_only) = payload.get("host_only").and_then(Value::as_bool) else {
                    return Vec::new();
                };
                if self.party.host_only == host_only {
                    return Vec::new();
                }
                self.party.host_only = host_only;
                if self.party.is_host() {
                    Vec::new()
                } else {
                    vec![PartyEffect::Event(PartyEvent::ControlChanged(host_only))]
                }
            }
            "member_joined" => vec![PartyEffect::Heartbeat {
                announce: text(payload, "member_id").map(str::to_owned),
            }],
            "member_left" => self.on_member_left(
                text(payload, "member_id"),
                text(payload, "reason"),
                text(payload, "host_member_id"),
            ),
            "ended" => vec![PartyEffect::Finish(PartyEvent::Ended)],
            "media" => {
                let Some(media) = payload.get("media").and_then(decode::<PartyMedia>) else {
                    return Vec::new();
                };
                let playback = payload
                    .get("playback")
                    .filter(|value| !value.is_null())
                    .and_then(decode::<PartyPlayback>);
                self.apply_media(media, playback, t)
            }
            "chat" => vec![PartyEffect::Chat(payload.clone())],
            _ => Vec::new(),
        }
    }

    pub fn on_presence(
        &mut self,
        joins: &[(String, Value)],
        leaves: &[String],
    ) -> Vec<PartyEffect> {
        let own = self.party.member_id.clone();
        let joined: HashSet<&str> = joins.iter().map(|(key, _)| key.as_str()).collect();
        let left: Vec<&String> = leaves
            .iter()
            .filter(|key| **key != own && !joined.contains(key.as_str()))
            .collect();
        for key in &left {
            self.party.presence.remove(key.as_str());
        }
        for (key, state) in joins {
            if *key == own || self.party.presence.contains_key(key) {
                continue;
            }
            let mut presence = decode::<PartyPresence>(state)
                .unwrap_or_else(|| PartyPresence::browsing(key));
            presence.member_id = key.clone();
            if presence.status == PartyMemberStatus::Ready {
                self.ignored_waiting.remove(key);
            }
            self.party.presence.insert(key.clone(), presence);
        }
        if left.is_empty() {
            Vec::new()
        } else {
            vec![PartyEffect::Send {
                event: "state_request",
                payload: json!({"member_id": own}),
            }]
        }
    }

    fn apply_status(&mut self, status: PartyPresence) -> Vec<PartyEffect> {
        if status.member_id == self.party.member_id {
            return Vec::new();
        }
        if status.status == PartyMemberStatus::Ready {
            self.ignored_waiting.remove(&status.member_id);
        }
        self.party.presence.insert(status.member_id.clone(), status);
        Vec::new()
    }

    fn on_member_left(
        &mut self,
        member_id: Option<&str>,
        reason: Option<&str>,
        host_member_id: Option<&str>,
    ) -> Vec<PartyEffect> {
        let Some(member_id) = member_id else {
            return Vec::new();
        };
        if member_id == self.party.member_id {
            return if reason == Some("removed") {
                vec![PartyEffect::Finish(PartyEvent::Removed)]
            } else {
                self.rejoin_or_finish()
            };
        }
        let name = self.party.member(member_id).map(|m| m.display_name.clone());
        self.party.members.retain(|member| member.id != member_id);
        self.party.presence.remove(member_id);
        if let Some(host) = host_member_id {
            self.party.host_member_id = Some(host.to_owned());
        }
        self.ignored_waiting.remove(member_id);
        name.map(|name| vec![PartyEffect::Event(PartyEvent::MemberLeft(name))])
            .unwrap_or_default()
    }

    fn rejoin_or_finish(&mut self) -> Vec<PartyEffect> {
        if self.rejoining {
            return Vec::new();
        }
        self.rejoining = true;
        vec![PartyEffect::Rejoin]
    }

    pub fn rejoin_failed(&mut self, error: Option<PartyError>) -> Vec<PartyEffect> {
        self.rejoining = false;
        let event = if error == Some(PartyError::Removed) {
            PartyEvent::Removed
        } else {
            PartyEvent::Ended
        };
        vec![PartyEffect::Finish(event)]
    }

    pub fn rejoined(&mut self, snapshot: &PartySnapshot) -> Option<ActiveParty> {
        self.rejoining = false;
        let info = snapshot.party.as_ref()?;
        let member_id = snapshot.member_id.clone()?;
        if member_id == self.party.member_id {
            self.party.members = snapshot.members.clone();
            self.party.host_member_id = info.host_member_id.clone();
            return None;
        }
        Some(active_party(snapshot)?)
    }

    pub fn apply_snapshot(
        &mut self,
        snapshot: &PartySnapshot,
        announce: Option<&str>,
        t: Time,
    ) -> Vec<PartyEffect> {
        match snapshot.error.as_deref() {
            None => {}
            Some("not_member") => return self.rejoin_or_finish(),
            Some("party_ended") => return vec![PartyEffect::Finish(PartyEvent::Ended)],
            Some(_) => return Vec::new(),
        }
        let Some(info) = &snapshot.party else {
            return Vec::new();
        };
        let ids: HashSet<&str> = snapshot.members.iter().map(|m| m.id.as_str()).collect();
        self.party.members = snapshot.members.clone();
        self.party.host_member_id = info.host_member_id.clone();
        self.party.max_members = info.max_members;
        self.party.host_only = info.host_only;
        self.party.presence.retain(|key, _| ids.contains(key.as_str()));
        let mut effects = Vec::new();
        if info.media.key() != self.party.media.key() {
            effects.extend(self.apply_media(info.media.clone(), info.playback.clone(), t));
        }
        if let Some(playback) = info.playback.clone() {
            effects.extend(self.accept(playback, t));
        }
        if let Some(id) = announce.filter(|id| *id != self.party.member_id)
            && let Some(member) = self.party.member(id)
        {
            effects.push(PartyEffect::Event(PartyEvent::MemberJoined(
                member.display_name.clone(),
            )));
        }
        effects
    }

    fn apply_media(
        &mut self,
        media: PartyMedia,
        playback: Option<PartyPlayback>,
        t: Time,
    ) -> Vec<PartyEffect> {
        let changed = media.key() != self.party.media.key();
        self.party.media = media.clone();
        if !changed {
            return playback
                .map(|playback| self.accept(playback, t))
                .unwrap_or_default();
        }
        let key = media.key();
        self.playback = playback.filter(|p| p.media_key.as_deref().is_none_or(|k| k == key));
        self.ignored_waiting.clear();
        self.wait_started = None;
        vec![PartyEffect::Event(PartyEvent::MediaChanged(media))]
    }

    fn accept(&mut self, candidate: PartyPlayback, t: Time) -> Vec<PartyEffect> {
        if candidate
            .media_key
            .as_deref()
            .is_some_and(|key| key != self.party.media.key())
        {
            return Vec::new();
        }
        if self.party.host_only
            && candidate.member_id.as_deref() != self.party.host_member_id.as_deref()
        {
            return Vec::new();
        }
        let current = self.playback.clone();
        if !is_newer(&candidate, current.as_ref()) {
            return Vec::new();
        }
        if let Some(current) = &current
            && current.paused
            && !candidate.paused
        {
            self.resumed_at = Some(t.local_ms);
            if current.waiting {
                let waiting = waiting_member_ids(&self.party, &HashSet::new());
                self.ignored_waiting.extend(waiting);
            }
        }
        self.playback = Some(candidate.clone());
        if candidate.member_id.as_deref() == Some(self.party.member_id.as_str()) {
            return Vec::new();
        }
        let Some(action) = action_of(current.as_ref(), &candidate) else {
            return Vec::new();
        };
        let Some(name) = candidate
            .member_id
            .as_deref()
            .and_then(|id| self.party.member(id))
            .map(|member| member.display_name.clone())
        else {
            return Vec::new();
        };
        vec![PartyEffect::Notice(PartyNotice {
            name,
            action,
            position_ms: candidate.position_ms,
            rate: candidate.rate,
        })]
    }

    fn next_playback(
        &self,
        paused: bool,
        position_ms: i64,
        rate: f32,
        waiting: bool,
        media_key: String,
        at: i64,
    ) -> PartyPlayback {
        PartyPlayback {
            seq: self.playback.as_ref().map_or(0, |p| p.seq) + 1,
            member_id: Some(self.party.member_id.clone()),
            paused,
            position_ms: position_ms.max(0),
            rate: rate.clamp(0.25, 4.0),
            at,
            media_key: Some(media_key),
            waiting,
        }
    }

    pub fn publish(
        &mut self,
        paused: bool,
        position_ms: i64,
        rate: f32,
        waiting: bool,
        at: i64,
        t: Time,
    ) -> Vec<PartyEffect> {
        if !self.party.can_control() {
            return Vec::new();
        }
        let next = self.next_playback(paused, position_ms, rate, waiting, self.party.media.key(), at);
        if let Some(current) = &self.playback
            && current.paused
            && !paused
        {
            self.resumed_at = Some(t.local_ms);
            if current.waiting && !waiting {
                let waiting = waiting_member_ids(&self.party, &HashSet::new());
                self.ignored_waiting.extend(waiting);
            }
        }
        if !waiting {
            self.wait_started = None;
        }
        self.persist_due = Some(t.local_ms + PERSIST_DELAY_MS);
        let effect = self.send_state(&next);
        self.playback = Some(next);
        vec![effect]
    }

    pub fn publish_media(&mut self, media: PartyMedia, position_ms: i64, t: Time) -> Vec<PartyEffect> {
        if media == self.party.media || !self.party.is_host() {
            return Vec::new();
        }
        let same_content = media.key() == self.party.media.key();
        self.party.media = media.clone();
        let next = if same_content {
            self.playback.clone()
        } else {
            let at = self.now_ms(t);
            let next = self.next_playback(false, position_ms, 1.0, false, media.key(), at);
            self.playback = Some(next.clone());
            Some(next)
        };
        self.persist_due = None;
        vec![PartyEffect::Persist {
            media: Some(media),
            playback: next,
        }]
    }

    pub fn set_host_only(&mut self, enabled: bool) -> Option<bool> {
        if !self.party.is_host() || self.party.host_only == enabled {
            return None;
        }
        self.party.host_only = enabled;
        Some(enabled)
    }

    pub fn set_local_status(
        &mut self,
        status: PartyMemberStatus,
        media_key: Option<String>,
        behind_ms: Option<i64>,
        t: Time,
    ) {
        let next = PartyPresence {
            member_id: self.party.member_id.clone(),
            status,
            media_key,
            behind_ms,
        };
        if self.local_presence.as_ref() == Some(&next) {
            return;
        }
        if status == PartyMemberStatus::Ready {
            self.ignored_waiting.remove(&self.party.member_id);
        }
        self.party.presence.insert(self.party.member_id.clone(), next.clone());
        self.local_presence = Some(next);
        self.status_due = Some(t.local_ms + STATUS_DELAY_MS);
    }

    pub fn tick(&mut self, t: Time) -> Vec<PartyEffect> {
        let mut effects = Vec::new();
        if self.status_due.is_some_and(|due| t.local_ms >= due) {
            self.status_due = None;
            effects.push(PartyEffect::Send {
                event: "status",
                payload: encode(&self.own_presence()),
            });
        }
        if self.persist_due.is_some_and(|due| t.local_ms >= due) {
            self.persist_due = None;
            if let Some(playback) = self.playback.clone() {
                effects.push(PartyEffect::Persist {
                    media: None,
                    playback: Some(playback),
                });
            }
        }
        if t.local_ms >= self.heartbeat_due {
            self.heartbeat_due = t.local_ms + HEARTBEAT_INTERVAL_MS;
            effects.push(PartyEffect::Heartbeat { announce: None });
        }
        effects.extend(self.coordinate(t));
        effects
    }

    fn coordinate(&mut self, t: Time) -> Vec<PartyEffect> {
        if !self.party.connected {
            return Vec::new();
        }
        if coordinator_id(&self.party).as_deref() != Some(self.party.member_id.as_str()) {
            self.wait_started = None;
            return Vec::new();
        }
        let Some(playback) = self.playback.clone() else {
            return Vec::new();
        };
        if playback
            .media_key
            .as_deref()
            .is_some_and(|key| key != self.party.media.key())
        {
            return Vec::new();
        }
        let waiting = waiting_member_ids(&self.party, &self.ignored_waiting);
        if !playback.paused {
            let grace_over = self
                .resumed_at
                .is_none_or(|at| t.local_ms - at > RESUME_GRACE_MS);
            if !waiting.is_empty() && grace_over {
                let now = self.now_ms(t);
                let position = expected_position_ms(&playback, now, 0);
                let effects = self.publish(true, position, playback.rate, true, now, t);
                self.wait_started = Some(t.local_ms);
                return effects;
            }
        } else if playback.waiting {
            let started = *self.wait_started.get_or_insert(t.local_ms);
            if waiting.is_empty() || t.local_ms - started > WAIT_LIMIT_MS {
                self.ignored_waiting.extend(waiting);
                let now = self.now_ms(t);
                return self.publish(false, playback.position_ms, playback.rate, false, now, t);
            }
        }
        Vec::new()
    }
}

pub fn active_party(snapshot: &PartySnapshot) -> Option<ActiveParty> {
    let info = snapshot.party.as_ref()?;
    Some(ActiveParty {
        id: info.id.clone(),
        code: info.code.clone(),
        member_id: snapshot.member_id.clone()?,
        host_member_id: info.host_member_id.clone(),
        members: snapshot.members.clone(),
        presence: Default::default(),
        media: info.media.clone(),
        max_members: info.max_members,
        connected: false,
        host_only: info.host_only,
    })
}

pub fn member_name(members: &[PartyMember], id: &str) -> Option<String> {
    members
        .iter()
        .find(|member| member.id == id)
        .map(|member| member.display_name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::sessions::party::PartyInfo;

    fn media() -> PartyMedia {
        PartyMedia {
            kind: "movie".into(),
            meta_id: "tt1".into(),
            meta_type: "movie".into(),
            video_id: None,
            season: None,
            episode: None,
            title: "Film".into(),
            episode_title: None,
            description: None,
            poster: None,
            background: None,
            logo: None,
            source: None,
        }
    }

    fn member(id: &str) -> PartyMember {
        PartyMember {
            id: id.into(),
            display_name: id.to_uppercase(),
            avatar_url: None,
            avatar_color: None,
        }
    }

    fn session(own: &str, host: &str) -> PartySession {
        let snapshot = PartySnapshot {
            party: Some(PartyInfo {
                id: "p".into(),
                code: "ABC234".into(),
                host_member_id: Some(host.into()),
                host_only: false,
                media: media(),
                playback: None,
                max_members: 3,
            }),
            members: vec![member("a"), member("b")],
            member_id: Some(own.into()),
            ..Default::default()
        };
        let mut session = PartySession::new(
            active_party(&snapshot).unwrap(),
            None,
            PartyClock::default(),
            None,
            0,
        );
        session.party.connected = true;
        session
    }

    fn at(ms: i64) -> Time {
        Time { local_ms: ms, wall_ms: ms }
    }

    fn state(seq: i64, by: &str, paused: bool) -> Value {
        json!({
            "seq": seq, "member_id": by, "paused": paused, "position_ms": 1000,
            "rate": 1.0, "at": 0, "media_key": media().key(), "waiting": false
        })
    }

    #[test]
    fn stale_state_is_ignored() {
        let mut s = session("a", "a");
        s.on_message("state", &state(5, "b", true), at(0));
        s.on_message("state", &state(4, "b", false), at(0));
        assert!(s.playback().unwrap().paused);
    }

    #[test]
    fn host_only_drops_guest_state_and_blocks_publish() {
        let mut s = session("b", "a");
        s.on_message("control", &json!({"host_only": true}), at(0));
        s.on_message("state", &state(1, "b", true), at(0));
        assert!(s.playback().is_none());
        assert!(s.publish(true, 0, 1.0, false, 0, at(0)).is_empty());
    }

    #[test]
    fn coordinator_pauses_for_a_loading_member_then_resumes() {
        let mut s = session("a", "a");
        s.on_message("state", &state(1, "a", false), at(0));
        s.on_message(
            "status",
            &json!({"member_id": "b", "status": "buffering", "media_key": media().key()}),
            at(0),
        );
        s.tick(at(10_000));
        assert!(s.playback().unwrap().paused && s.playback().unwrap().waiting);
        s.on_message(
            "status",
            &json!({"member_id": "b", "status": "ready", "media_key": media().key()}),
            at(10_000),
        );
        s.tick(at(10_500));
        assert!(!s.playback().unwrap().paused);
    }

    #[test]
    fn only_the_coordinator_pauses_the_room() {
        let mut s = session("b", "a");
        s.on_presence(&[("a".into(), json!({"member_id": "a"}))], &[]);
        s.on_message("state", &state(1, "a", false), at(0));
        s.on_message(
            "status",
            &json!({"member_id": "a", "status": "buffering", "media_key": media().key()}),
            at(0),
        );
        s.tick(at(10_000));
        assert!(!s.playback().unwrap().paused);
    }

    #[test]
    fn host_leaving_hands_coordination_to_next_online_member() {
        let mut s = session("b", "a");
        s.on_message(
            "member_left",
            &json!({"member_id": "a", "host_member_id": "b"}),
            at(0),
        );
        assert!(s.party().is_host());
        assert_eq!(coordinator_id(s.party()).as_deref(), Some("b"));
    }
}
