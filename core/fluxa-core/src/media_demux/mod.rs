//! In-browser-capable container remuxing: parse MKV (Matroska), repackage
//! into WebM for MediaSource Extensions, without re-encoding.
//!
//! `ebml` is dependency-free and also used natively by
//! `fluxa-streaming-engine`'s Dolby Vision RPU rewriter and chapter
//! extractor (via the `streaming-shared` feature), so it stays available
//! outside the wasm/browser-only parts of this module.

pub mod ebml;

pub mod fmp4_mux;
pub mod mkv_demux;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod webm_mux;

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub fn remux_mkv_to_webm(mkv_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let demuxed = mkv_demux::demux(mkv_bytes).map_err(|e| format!("{e:?}"))?;
    Ok(webm_mux::remux(&demuxed))
}

/// Incremental MKV -> WebM remux session: feed it chunks as they arrive
/// (e.g. from a `fetch()` `ReadableStream`) and it hands back WebM bytes to
/// append to a `SourceBuffer` as soon as they're available — playback can
/// start well before the whole source file has downloaded. See
/// `mkv_demux::IncrementalDemuxer` and `webm_mux::ClusterWriter` docs for how
/// the two halves compose.
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
#[derive(Default)]
pub struct IncrementalRemuxSession {
    demuxer: mkv_demux::IncrementalDemuxer,
    clusters: webm_mux::ClusterWriter,
    init_written: bool,
}

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
impl IncrementalRemuxSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns WebM bytes to append next (may be empty if nothing new is
    /// ready yet — e.g. still buffering the Tracks element or a Cluster).
    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        let step = self.demuxer.push(chunk);
        self.drain(step)
    }

    /// Call once at end-of-stream; flushes the final still-open Cluster.
    pub fn finish(&mut self) -> Vec<u8> {
        let step = self.demuxer.flush();
        let mut out = self.drain(step);
        out.extend_from_slice(&self.clusters.finish());
        out
    }

    fn drain(&mut self, step: mkv_demux::IncrementalStep) -> Vec<u8> {
        let mut out = Vec::new();
        if step.tracks_ready && !self.init_written {
            let video = self
                .demuxer
                .tracks
                .iter()
                .find(|t| t.kind == mkv_demux::TrackKind::Video);
            let audio = self
                .demuxer
                .tracks
                .iter()
                .find(|t| t.kind == mkv_demux::TrackKind::Audio);
            out.extend_from_slice(&webm_mux::write_init(
                self.demuxer.timestamp_scale,
                video,
                audio,
            ));
            self.init_written = true;
        }
        if self.init_written && !step.packets.is_empty() {
            let video = self
                .demuxer
                .tracks
                .iter()
                .find(|t| t.kind == mkv_demux::TrackKind::Video);
            let audio = self
                .demuxer
                .tracks
                .iter()
                .find(|t| t.kind == mkv_demux::TrackKind::Audio);
            out.extend_from_slice(&self.clusters.push_packets(&step.packets, video, audio));
        }
        out
    }
}

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
#[cfg(test)]
mod tests {
    use super::*;
    use ebml::write_ebml_element;

    #[test]
    fn empty_input_reports_no_segment() {
        let err = mkv_demux::demux(&[]);
        assert!(err.is_err());
    }

    fn build_synthetic_mkv() -> Vec<u8> {
        const SEGMENT: u64 = 0x1853_8067;
        const INFO: u64 = 0x1549_A966;
        const TIMESTAMP_SCALE: u64 = 0x2AD7_B1;
        const TRACKS: u64 = 0x1654_AE6B;
        const TRACK_ENTRY: u64 = 0xAE;
        const TRACK_NUMBER: u64 = 0xD7;
        const TRACK_TYPE: u64 = 0x83;
        const CODEC_ID: u64 = 0x86;
        const VIDEO: u64 = 0xE0;
        const PIXEL_WIDTH: u64 = 0xB0;
        const PIXEL_HEIGHT: u64 = 0xBA;
        const AUDIO: u64 = 0xE1;
        const SAMPLING_FREQUENCY: u64 = 0xB5;
        const CHANNELS: u64 = 0x9F;
        const CLUSTER: u64 = 0x1F43_B675;
        const TIMECODE: u64 = 0xE7;
        const SIMPLE_BLOCK: u64 = 0xA3;

        let mut info = Vec::new();
        write_ebml_element(&mut info, TIMESTAMP_SCALE, &[0x0F, 0x42, 0x40]); // 1_000_000

        let mut video_entry = Vec::new();
        write_ebml_element(&mut video_entry, TRACK_NUMBER, &[1]);
        write_ebml_element(&mut video_entry, TRACK_TYPE, &[1]);
        write_ebml_element(&mut video_entry, CODEC_ID, b"V_VP9");
        let mut video_dims = Vec::new();
        write_ebml_element(&mut video_dims, PIXEL_WIDTH, &[0x05, 0x00]); // 1280
        write_ebml_element(&mut video_dims, PIXEL_HEIGHT, &[0x02, 0xD0]); // 720
        write_ebml_element(&mut video_entry, VIDEO, &video_dims);

        let mut audio_entry = Vec::new();
        write_ebml_element(&mut audio_entry, TRACK_NUMBER, &[2]);
        write_ebml_element(&mut audio_entry, TRACK_TYPE, &[2]);
        write_ebml_element(&mut audio_entry, CODEC_ID, b"A_OPUS");
        let mut audio_params = Vec::new();
        write_ebml_element(
            &mut audio_params,
            SAMPLING_FREQUENCY,
            &48_000f64.to_be_bytes(),
        );
        write_ebml_element(&mut audio_params, CHANNELS, &[2]);
        write_ebml_element(&mut audio_entry, AUDIO, &audio_params);

        let mut tracks = Vec::new();
        write_ebml_element(&mut tracks, TRACK_ENTRY, &video_entry);
        write_ebml_element(&mut tracks, TRACK_ENTRY, &audio_entry);

        fn simple_block(out: &mut Vec<u8>, track: u8, rel_ts: i16, keyframe: bool, data: &[u8]) {
            let mut payload = vec![0x80 | track];
            payload.extend_from_slice(&rel_ts.to_be_bytes());
            payload.push(if keyframe { 0x80 } else { 0x00 });
            payload.extend_from_slice(data);
            write_ebml_element(out, 0xA3, &payload);
        }

        let mut cluster = Vec::new();
        write_ebml_element(&mut cluster, TIMECODE, &[0]);
        simple_block(&mut cluster, 1, 0, true, &[0xAA, 0xBB, 0xCC]);
        simple_block(&mut cluster, 2, 0, true, &[0x11, 0x22]);
        simple_block(&mut cluster, 1, 40, false, &[0xDD, 0xEE]);
        let _ = SIMPLE_BLOCK;
        let mut cluster_elem = Vec::new();
        write_ebml_element(&mut cluster_elem, CLUSTER, &cluster);

        let mut segment_body = Vec::new();
        write_ebml_element(&mut segment_body, INFO, &info);
        write_ebml_element(&mut segment_body, TRACKS, &tracks);
        segment_body.extend_from_slice(&cluster_elem);

        let mut out = Vec::new();
        write_ebml_element(&mut out, SEGMENT, &segment_body);
        out
    }

    #[test]
    fn demux_reads_tracks_and_packets() {
        let mkv = build_synthetic_mkv();
        let demuxed = mkv_demux::demux(&mkv).expect("demux should succeed");
        assert_eq!(demuxed.timestamp_scale, 1_000_000);
        assert_eq!(demuxed.tracks.len(), 2);
        assert_eq!(demuxed.packets.len(), 3);
        let video_track = demuxed
            .tracks
            .iter()
            .find(|t| t.kind == mkv_demux::TrackKind::Video)
            .unwrap();
        assert_eq!(video_track.codec_id, "V_VP9");
        assert_eq!(video_track.width, Some(1280));
        let audio_track = demuxed
            .tracks
            .iter()
            .find(|t| t.kind == mkv_demux::TrackKind::Audio)
            .unwrap();
        assert_eq!(audio_track.codec_id, "A_OPUS");
        assert_eq!(audio_track.sampling_frequency, Some(48_000.0));
    }

    #[test]
    fn remux_round_trips_through_webm() {
        let mkv = build_synthetic_mkv();
        let webm = remux_mkv_to_webm(&mkv).expect("remux should succeed");

        // The muxed output must itself be a valid Matroska-family stream our
        // own demuxer can read back — same tracks, same packet count/data.
        let redemuxed = mkv_demux::demux(&webm).expect("remuxed output should redemux");
        assert_eq!(redemuxed.tracks.len(), 2);
        assert_eq!(redemuxed.packets.len(), 3);
        assert_eq!(redemuxed.packets[0].data, vec![0xAA, 0xBB, 0xCC]);
        assert_eq!(redemuxed.packets[1].data, vec![0x11, 0x22]);
        assert_eq!(redemuxed.packets[2].data, vec![0xDD, 0xEE]);
        assert!(redemuxed.packets[0].keyframe);
        assert!(!redemuxed.packets[2].keyframe);
    }

    #[test]
    fn incremental_demux_matches_whole_buffer_when_fed_byte_by_byte() {
        let mkv = build_synthetic_mkv();
        let whole = mkv_demux::demux(&mkv).expect("whole-buffer demux should succeed");

        let mut incremental = mkv_demux::IncrementalDemuxer::new();
        let mut packets = Vec::new();
        let mut saw_tracks_ready = false;
        for byte in &mkv {
            let step = incremental.push(std::slice::from_ref(byte));
            saw_tracks_ready |= step.tracks_ready;
            packets.extend(step.packets);
        }
        let tail = incremental.flush();
        saw_tracks_ready |= tail.tracks_ready;
        packets.extend(tail.packets);

        assert!(
            saw_tracks_ready,
            "tracks should become available before EOF"
        );
        assert_eq!(incremental.tracks.len(), whole.tracks.len());
        assert_eq!(incremental.timestamp_scale, whole.timestamp_scale);
        assert_eq!(packets.len(), whole.packets.len());
        for (a, b) in packets.iter().zip(whole.packets.iter()) {
            assert_eq!(a.track_number, b.track_number);
            assert_eq!(a.data, b.data);
            assert_eq!(a.keyframe, b.keyframe);
        }
    }

    #[test]
    fn incremental_demux_handles_unknown_size_cluster_only_at_eof() {
        const SEGMENT: u64 = 0x1853_8067;
        const TRACKS: u64 = 0x1654_AE6B;
        const TRACK_ENTRY: u64 = 0xAE;
        const TRACK_NUMBER: u64 = 0xD7;
        const TRACK_TYPE: u64 = 0x83;
        const CODEC_ID: u64 = 0x86;
        const CLUSTER: u64 = 0x1F43_B675;
        const TIMECODE: u64 = 0xE7;

        let mut video_entry = Vec::new();
        write_ebml_element(&mut video_entry, TRACK_NUMBER, &[1]);
        write_ebml_element(&mut video_entry, TRACK_TYPE, &[1]);
        write_ebml_element(&mut video_entry, CODEC_ID, b"V_VP9");
        let mut tracks = Vec::new();
        write_ebml_element(&mut tracks, TRACK_ENTRY, &video_entry);

        let mut cluster_body = Vec::new();
        write_ebml_element(&mut cluster_body, TIMECODE, &[0]);
        let mut block_payload = vec![0x81u8]; // track number 1
        block_payload.extend_from_slice(&0i16.to_be_bytes());
        block_payload.push(0x80);
        block_payload.extend_from_slice(&[0xCA, 0xFE]);
        write_ebml_element(&mut cluster_body, 0xA3, &block_payload);

        let mut segment_body = Vec::new();
        write_ebml_element(&mut segment_body, TRACKS, &tracks);
        // Unknown-size Cluster with no following sibling — its end can only
        // be known at EOF.
        ebml::write_ebml_id(&mut segment_body, CLUSTER);
        segment_body.extend_from_slice(&[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        segment_body.extend_from_slice(&cluster_body);

        let mut mkv = Vec::new();
        write_ebml_element(&mut mkv, SEGMENT, &segment_body);

        let mut incremental = mkv_demux::IncrementalDemuxer::new();
        let before_eof = incremental.push(&mkv);
        assert!(
            before_eof.packets.is_empty(),
            "unknown-size cluster with no following sibling must not be parsed before EOF"
        );

        let at_eof = incremental.flush();
        assert_eq!(at_eof.packets.len(), 1);
        assert_eq!(at_eof.packets[0].data, vec![0xCA, 0xFE]);
    }

    #[test]
    fn incremental_remux_session_produces_playable_webm_fed_in_small_chunks() {
        let mkv = build_synthetic_mkv();
        let mut session = IncrementalRemuxSession::new();
        let mut webm = Vec::new();
        for chunk in mkv.chunks(7) {
            webm.extend_from_slice(&session.push(chunk));
        }
        webm.extend_from_slice(&session.finish());

        // What SourceBuffer.appendBuffer would have received, concatenated,
        // must itself be a valid, redemuxable WebM stream — same content as
        // the whole-buffer `remux_mkv_to_webm` path.
        let redemuxed = mkv_demux::demux(&webm).expect("incrementally muxed output should redemux");
        assert_eq!(redemuxed.tracks.len(), 2);
        assert_eq!(redemuxed.packets.len(), 3);
        assert_eq!(redemuxed.packets[0].data, vec![0xAA, 0xBB, 0xCC]);
        assert_eq!(redemuxed.packets[1].data, vec![0x11, 0x22]);
        assert_eq!(redemuxed.packets[2].data, vec![0xDD, 0xEE]);
    }
}

/// Incremental MKV -> fragmented MP4 remux session, the AVFoundation-facing
/// counterpart to [`IncrementalRemuxSession`]. Feed it the source bytes as
/// they arrive and it hands back an fMP4 byte stream an AVPlayer can start
/// playing before the download finishes.
#[derive(Default)]
pub struct IncrementalFmp4Session {
    demuxer: mkv_demux::IncrementalDemuxer,
    fragments: Option<fmp4_mux::FragmentWriter>,
}

impl IncrementalFmp4Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        let step = self.demuxer.push(chunk);
        self.drain(step)
    }

    pub fn finish(&mut self) -> Vec<u8> {
        let step = self.demuxer.flush();
        let mut out = self.drain(step);
        if let Some(writer) = self.fragments.as_mut() {
            if let Some(fragment) = writer.flush() {
                out.extend_from_slice(&fragment);
            }
        }
        out
    }

    /// `None` until the source's Tracks element has been parsed; afterwards,
    /// whether the tracks are ones we can copy into MP4 without re-encoding.
    pub fn is_supported(&self) -> Option<bool> {
        if self.demuxer.tracks.is_empty() {
            return None;
        }
        Some(
            self.demuxer.tracks.iter().any(|track| {
                track.kind == mkv_demux::TrackKind::Video && fmp4_mux::supports(track)
            }),
        )
    }

    fn drain(&mut self, step: mkv_demux::IncrementalStep) -> Vec<u8> {
        let mut out = Vec::new();
        if step.tracks_ready && self.fragments.is_none() {
            let video = self.demuxer.tracks.iter().find(|track| {
                track.kind == mkv_demux::TrackKind::Video && fmp4_mux::supports(track)
            });
            let audio = self.demuxer.tracks.iter().find(|track| {
                track.kind == mkv_demux::TrackKind::Audio && fmp4_mux::supports(track)
            });
            if video.is_none() {
                return out;
            }
            out.extend_from_slice(&fmp4_mux::write_init(
                self.demuxer.timestamp_scale,
                video,
                audio,
            ));
            self.fragments = Some(fmp4_mux::FragmentWriter::new(
                self.demuxer.timestamp_scale,
                video,
                audio,
            ));
        }
        if let Some(writer) = self.fragments.as_mut() {
            for packet in &step.packets {
                if let Some(fragment) = writer.push(packet) {
                    out.extend_from_slice(&fragment);
                }
            }
        }
        out
    }
}
