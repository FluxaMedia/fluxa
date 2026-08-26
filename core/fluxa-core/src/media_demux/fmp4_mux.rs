//! Remuxes supported demuxed MKV tracks/packets into fragmented MP4, the
//! container AVFoundation accepts. Video is bitstream-copy only; codec
//! configuration and Dolby Vision signaling still need source-specific
//! handling before this is suitable for every HDR/DV file.

use super::mkv_demux::{DemuxResult, Packet, Track, TrackKind};

const CODEC_AVC: &str = "V_MPEG4/ISO/AVC";
const CODEC_HEVC: &str = "V_MPEGH/ISO/HEVC";
const CODEC_AAC: &str = "A_AAC";

pub fn supports(track: &Track) -> bool {
    matches!(track.codec_id.as_str(), CODEC_AVC | CODEC_HEVC | CODEC_AAC)
        && !track.codec_private.is_empty()
}

pub fn remux(demuxed: &DemuxResult) -> Option<Vec<u8>> {
    let video = demuxed
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Video && supports(track));
    let audio = demuxed
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Audio && supports(track));
    // An audio-only fMP4 is not a useful fallback for a video playback
    // request. In particular, do not turn a VP9+AAC file into an AAC-only
    // output and report it as successfully adapted.
    if video.is_none() {
        return None;
    }

    let mut out = write_init(demuxed.timestamp_scale, video, audio);
    let mut writer = FragmentWriter::new(demuxed.timestamp_scale, video, audio);
    for packet in &demuxed.packets {
        if let Some(fragment) = writer.push(packet) {
            out.extend_from_slice(&fragment);
        }
    }
    if let Some(fragment) = writer.flush() {
        out.extend_from_slice(&fragment);
    }
    Some(out)
}

pub fn timescale_for(timestamp_scale: u64) -> u32 {
    let scale = if timestamp_scale == 0 {
        1_000_000
    } else {
        timestamp_scale
    };
    (1_000_000_000u64 / scale).max(1) as u32
}

pub fn write_init(timestamp_scale: u64, video: Option<&Track>, audio: Option<&Track>) -> Vec<u8> {
    let timescale = timescale_for(timestamp_scale);
    let mut out = Vec::new();
    out.extend_from_slice(&ftyp());

    let mut moov = Vec::new();
    moov.extend_from_slice(&mvhd(timescale));
    let mut track_id = 1u32;
    if let Some(track) = video {
        moov.extend_from_slice(&trak(track_id, timescale, track));
        track_id += 1;
    }
    if let Some(track) = audio {
        moov.extend_from_slice(&trak(track_id, timescale, track));
    }

    let mut mvex = Vec::new();
    let count = video.is_some() as u32 + audio.is_some() as u32;
    for id in 1..=count {
        mvex.extend_from_slice(&trex(id));
    }
    moov.extend_from_slice(&boxed(b"mvex", &mvex));
    out.extend_from_slice(&boxed(b"moov", &moov));
    out
}

pub struct FragmentWriter {
    timescale: u32,
    video_track: Option<u64>,
    video_codec: Option<String>,
    audio_track: Option<u64>,
    video_id: u32,
    audio_id: u32,
    sequence: u32,
    pending: Vec<Packet>,
    fragment_start: Option<i64>,
    video_decode_time: u64,
    audio_decode_time: u64,
    max_span: i64,
}

impl FragmentWriter {
    pub fn new(timestamp_scale: u64, video: Option<&Track>, audio: Option<&Track>) -> Self {
        let timescale = timescale_for(timestamp_scale);
        let mut next_id = 1u32;
        let video_id = if video.is_some() {
            let id = next_id;
            next_id += 1;
            id
        } else {
            0
        };
        let audio_id = if audio.is_some() { next_id } else { 0 };
        Self {
            timescale,
            video_track: video.map(|track| track.number),
            video_codec: video.map(|track| track.codec_id.clone()),
            audio_track: audio.map(|track| track.number),
            video_id,
            audio_id,
            sequence: 1,
            pending: Vec::new(),
            fragment_start: None,
            video_decode_time: 0,
            audio_decode_time: 0,
            max_span: i64::from(timescale) * 2,
        }
    }

    pub fn push(&mut self, packet: &Packet) -> Option<Vec<u8>> {
        if Some(packet.track_number) != self.video_track
            && Some(packet.track_number) != self.audio_track
        {
            return None;
        }

        let mut emitted = None;
        let is_video = Some(packet.track_number) == self.video_track;
        let starts_fragment = is_video && packet.keyframe;
        if let Some(start) = self.fragment_start {
            let span = packet.timestamp - start;
            if (starts_fragment && !self.pending.is_empty()) || span >= self.max_span {
                emitted = self.flush();
            }
        }
        if self.fragment_start.is_none() {
            self.fragment_start = Some(packet.timestamp);
        }
        let mut packet = packet.clone();
        if is_video && matches!(self.video_codec.as_deref(), Some(CODEC_AVC | CODEC_HEVC)) {
            // Matroska stores AVC/HEVC blocks as Annex-B in the common case;
            // ISO-BMFF sample data must use length-prefixed NAL units.
            packet.data = annex_b_to_length_prefixed(&packet.data);
        }
        self.pending.push(packet);
        emitted
    }

    pub fn flush(&mut self) -> Option<Vec<u8>> {
        if self.pending.is_empty() {
            return None;
        }
        let pending = std::mem::take(&mut self.pending);
        self.fragment_start = None;

        let video: Vec<&Packet> = pending
            .iter()
            .filter(|packet| Some(packet.track_number) == self.video_track)
            .collect();
        let audio: Vec<&Packet> = pending
            .iter()
            .filter(|packet| Some(packet.track_number) == self.audio_track)
            .collect();

        let mut runs = Vec::new();
        if !video.is_empty() {
            runs.push((self.video_id, self.video_decode_time, video));
        }
        if !audio.is_empty() {
            runs.push((self.audio_id, self.audio_decode_time, audio));
        }

        let mut moof_body = Vec::new();
        moof_body.extend_from_slice(&mfhd(self.sequence));

        let header_len: usize = 8 + moof_body.len() + runs.iter().map(traf_len).sum::<usize>();
        let mut data_offset = header_len + 8;
        let mut mdat = Vec::new();

        for (track_id, decode_time, packets) in &runs {
            let durations = sample_durations(packets, self.timescale);
            moof_body.extend_from_slice(&traf(
                *track_id,
                *decode_time,
                packets,
                &durations,
                data_offset as u32,
            ));
            for packet in packets {
                mdat.extend_from_slice(&packet.data);
                data_offset += packet.data.len();
            }
            let total: u64 = durations.iter().map(|value| u64::from(*value)).sum();
            if *track_id == self.video_id {
                self.video_decode_time += total;
            } else {
                self.audio_decode_time += total;
            }
        }

        let mut out = boxed(b"moof", &moof_body);
        out.extend_from_slice(&boxed(b"mdat", &mdat));
        self.sequence += 1;
        Some(out)
    }
}

fn sample_durations(packets: &[&Packet], timescale: u32) -> Vec<u32> {
    let fallback = timescale / 25;
    let mut durations = Vec::with_capacity(packets.len());
    for index in 0..packets.len() {
        let duration = if index + 1 < packets.len() {
            (packets[index + 1].timestamp - packets[index].timestamp).max(0) as u32
        } else {
            durations.last().copied().unwrap_or(fallback)
        };
        durations.push(if duration == 0 { fallback } else { duration });
    }
    durations
}

fn annex_b_to_length_prefixed(data: &[u8]) -> Vec<u8> {
    let Some(mut start) = start_code_at(data, 0) else {
        return data.to_vec();
    };
    let mut out = Vec::with_capacity(data.len());
    while start < data.len() {
        let nal_start = start + start_code_len(data, start);
        let next = find_start_code(data, nal_start).unwrap_or(data.len());
        if next > nal_start {
            let nal = &data[nal_start..next];
            out.extend_from_slice(&(nal.len() as u32).to_be_bytes());
            out.extend_from_slice(nal);
        }
        if next == data.len() {
            break;
        }
        start = next;
    }
    if out.is_empty() { data.to_vec() } else { out }
}

fn find_start_code(data: &[u8], from: usize) -> Option<usize> {
    (from..data.len()).find(|&index| start_code_at(data, index).is_some())
}

fn start_code_at(data: &[u8], index: usize) -> Option<usize> {
    if data.get(index..index + 3) == Some(&[0, 0, 1]) {
        Some(3)
    } else if data.get(index..index + 4) == Some(&[0, 0, 0, 1]) {
        Some(4)
    } else {
        None
    }
}

fn start_code_len(data: &[u8], index: usize) -> usize {
    start_code_at(data, index).unwrap_or(0)
}

fn traf_len(run: &(u32, u64, Vec<&Packet>)) -> usize {
    8 + 16 + 20 + (20 + run.2.len() * 12)
}

fn traf(
    track_id: u32,
    decode_time: u64,
    packets: &[&Packet],
    durations: &[u32],
    data_offset: u32,
) -> Vec<u8> {
    let mut body = Vec::new();

    let mut tfhd = Vec::new();
    tfhd.extend_from_slice(&[0, 0x02, 0, 0x00]);
    tfhd.extend_from_slice(&track_id.to_be_bytes());
    body.extend_from_slice(&boxed(b"tfhd", &tfhd));

    let mut tfdt = Vec::new();
    tfdt.extend_from_slice(&[1, 0, 0, 0]);
    tfdt.extend_from_slice(&decode_time.to_be_bytes());
    body.extend_from_slice(&boxed(b"tfdt", &tfdt));

    let mut trun = Vec::new();
    trun.extend_from_slice(&[0, 0x00, 0x07, 0x01]);
    trun.extend_from_slice(&(packets.len() as u32).to_be_bytes());
    trun.extend_from_slice(&data_offset.to_be_bytes());
    for (packet, duration) in packets.iter().zip(durations) {
        trun.extend_from_slice(&duration.to_be_bytes());
        trun.extend_from_slice(&(packet.data.len() as u32).to_be_bytes());
        let flags: u32 = if packet.keyframe {
            0x0200_0000
        } else {
            0x0101_0000
        };
        trun.extend_from_slice(&flags.to_be_bytes());
    }
    body.extend_from_slice(&boxed(b"trun", &trun));

    boxed(b"traf", &body)
}

fn mfhd(sequence: u32) -> Vec<u8> {
    let mut body = vec![0, 0, 0, 0];
    body.extend_from_slice(&sequence.to_be_bytes());
    boxed(b"mfhd", &body)
}

fn ftyp() -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(b"isom");
    body.extend_from_slice(&512u32.to_be_bytes());
    for brand in [b"isom", b"iso6", b"iso2", b"avc1", b"mp41"] {
        body.extend_from_slice(brand);
    }
    boxed(b"ftyp", &body)
}

fn mvhd(timescale: u32) -> Vec<u8> {
    let mut body = vec![0, 0, 0, 0];
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&timescale.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    body.extend_from_slice(&0x0100u16.to_be_bytes());
    body.extend_from_slice(&[0; 10]);
    body.extend_from_slice(&unity_matrix());
    body.extend_from_slice(&[0; 24]);
    body.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes());
    boxed(b"mvhd", &body)
}

fn trak(track_id: u32, timescale: u32, track: &Track) -> Vec<u8> {
    let mut body = Vec::new();

    let mut tkhd = vec![0, 0, 0, 3];
    tkhd.extend_from_slice(&0u32.to_be_bytes());
    tkhd.extend_from_slice(&0u32.to_be_bytes());
    tkhd.extend_from_slice(&track_id.to_be_bytes());
    tkhd.extend_from_slice(&0u32.to_be_bytes());
    tkhd.extend_from_slice(&0u32.to_be_bytes());
    tkhd.extend_from_slice(&[0; 8]);
    tkhd.extend_from_slice(&0u16.to_be_bytes());
    tkhd.extend_from_slice(&0u16.to_be_bytes());
    let volume: u16 = if track.kind == TrackKind::Audio {
        0x0100
    } else {
        0
    };
    tkhd.extend_from_slice(&volume.to_be_bytes());
    tkhd.extend_from_slice(&0u16.to_be_bytes());
    tkhd.extend_from_slice(&unity_matrix());
    let width = track.width.unwrap_or(0) as u32;
    let height = track.height.unwrap_or(0) as u32;
    tkhd.extend_from_slice(&(width << 16).to_be_bytes());
    tkhd.extend_from_slice(&(height << 16).to_be_bytes());
    body.extend_from_slice(&boxed(b"tkhd", &tkhd));

    let mut mdia = Vec::new();
    let mut mdhd = vec![0, 0, 0, 0];
    mdhd.extend_from_slice(&0u32.to_be_bytes());
    mdhd.extend_from_slice(&0u32.to_be_bytes());
    mdhd.extend_from_slice(&timescale.to_be_bytes());
    mdhd.extend_from_slice(&0u32.to_be_bytes());
    mdhd.extend_from_slice(&0x55C4u16.to_be_bytes());
    mdhd.extend_from_slice(&0u16.to_be_bytes());
    mdia.extend_from_slice(&boxed(b"mdhd", &mdhd));

    let handler: &[u8; 4] = if track.kind == TrackKind::Video {
        b"vide"
    } else {
        b"soun"
    };
    let mut hdlr = vec![0, 0, 0, 0];
    hdlr.extend_from_slice(&0u32.to_be_bytes());
    hdlr.extend_from_slice(handler);
    hdlr.extend_from_slice(&[0; 12]);
    hdlr.extend_from_slice(b"Fluxa\0");
    mdia.extend_from_slice(&boxed(b"hdlr", &hdlr));

    let mut minf = Vec::new();
    if track.kind == TrackKind::Video {
        minf.extend_from_slice(&boxed(b"vmhd", &[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]));
    } else {
        minf.extend_from_slice(&boxed(b"smhd", &[0, 0, 0, 0, 0, 0, 0, 0]));
    }

    let mut dref = vec![0, 0, 0, 0];
    dref.extend_from_slice(&1u32.to_be_bytes());
    dref.extend_from_slice(&boxed(b"url ", &[0, 0, 0, 1]));
    minf.extend_from_slice(&boxed(b"dinf", &boxed(b"dref", &dref)));

    let mut stbl = Vec::new();
    let mut stsd = vec![0, 0, 0, 0];
    stsd.extend_from_slice(&1u32.to_be_bytes());
    stsd.extend_from_slice(&sample_entry(track));
    stbl.extend_from_slice(&boxed(b"stsd", &stsd));
    stbl.extend_from_slice(&boxed(b"stts", &[0, 0, 0, 0, 0, 0, 0, 0]));
    stbl.extend_from_slice(&boxed(b"stsc", &[0, 0, 0, 0, 0, 0, 0, 0]));
    stbl.extend_from_slice(&boxed(b"stsz", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    stbl.extend_from_slice(&boxed(b"stco", &[0, 0, 0, 0, 0, 0, 0, 0]));
    minf.extend_from_slice(&boxed(b"stbl", &stbl));

    mdia.extend_from_slice(&boxed(b"minf", &minf));
    body.extend_from_slice(&boxed(b"mdia", &mdia));
    boxed(b"trak", &body)
}

fn sample_entry(track: &Track) -> Vec<u8> {
    match track.codec_id.as_str() {
        CODEC_AVC => visual_entry(b"avc1", track, &boxed(b"avcC", &track.codec_private)),
        CODEC_HEVC => visual_entry(b"hvc1", track, &boxed(b"hvcC", &track.codec_private)),
        CODEC_AAC => audio_entry(b"mp4a", track, &boxed(b"esds", &esds(&track.codec_private))),
        _ => Vec::new(),
    }
}

fn visual_entry(kind: &[u8; 4], track: &Track, config: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&[0; 6]);
    body.extend_from_slice(&1u16.to_be_bytes());
    body.extend_from_slice(&[0; 16]);
    body.extend_from_slice(&(track.width.unwrap_or(0) as u16).to_be_bytes());
    body.extend_from_slice(&(track.height.unwrap_or(0) as u16).to_be_bytes());
    body.extend_from_slice(&0x0048_0000u32.to_be_bytes());
    body.extend_from_slice(&0x0048_0000u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&1u16.to_be_bytes());
    body.extend_from_slice(&[0; 32]);
    body.extend_from_slice(&0x0018u16.to_be_bytes());
    body.extend_from_slice(&0xFFFFu16.to_be_bytes());
    body.extend_from_slice(config);
    boxed(kind, &body)
}

fn audio_entry(kind: &[u8; 4], track: &Track, config: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&[0; 6]);
    body.extend_from_slice(&1u16.to_be_bytes());
    body.extend_from_slice(&[0; 8]);
    body.extend_from_slice(&(track.channels.unwrap_or(2) as u16).to_be_bytes());
    body.extend_from_slice(&16u16.to_be_bytes());
    body.extend_from_slice(&0u16.to_be_bytes());
    body.extend_from_slice(&0u16.to_be_bytes());
    let rate = track.sampling_frequency.unwrap_or(48000.0) as u32;
    body.extend_from_slice(&(rate << 16).to_be_bytes());
    body.extend_from_slice(config);
    boxed(kind, &body)
}

fn esds(specific_config: &[u8]) -> Vec<u8> {
    let mut decoder_specific = vec![0x05];
    decoder_specific.push(specific_config.len() as u8);
    decoder_specific.extend_from_slice(specific_config);

    let mut decoder_config = vec![0x04];
    decoder_config.push((13 + decoder_specific.len()) as u8);
    decoder_config.push(0x40);
    decoder_config.push(0x15);
    decoder_config.extend_from_slice(&[0, 0, 0]);
    decoder_config.extend_from_slice(&0u32.to_be_bytes());
    decoder_config.extend_from_slice(&0u32.to_be_bytes());
    decoder_config.extend_from_slice(&decoder_specific);

    let sl_config = vec![0x06, 0x01, 0x02];

    let mut es = vec![0x03];
    es.push((3 + decoder_config.len() + sl_config.len()) as u8);
    es.extend_from_slice(&1u16.to_be_bytes());
    es.push(0);
    es.extend_from_slice(&decoder_config);
    es.extend_from_slice(&sl_config);

    let mut body = vec![0, 0, 0, 0];
    body.extend_from_slice(&es);
    body
}

fn unity_matrix() -> [u8; 36] {
    let mut matrix = [0u8; 36];
    matrix[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    matrix[16..20].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    matrix[32..36].copy_from_slice(&0x4000_0000u32.to_be_bytes());
    matrix
}

fn trex(track_id: u32) -> Vec<u8> {
    let mut body = vec![0, 0, 0, 0];
    body.extend_from_slice(&track_id.to_be_bytes());
    body.extend_from_slice(&1u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    boxed(b"trex", &body)
}

fn boxed(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + body.len());
    out.extend_from_slice(&((8 + body.len()) as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tracks() -> (Track, Track) {
        (
            Track {
                number: 1,
                kind: TrackKind::Video,
                codec_id: CODEC_AVC.to_string(),
                codec_private: vec![
                    1, 0x64, 0, 0x1F, 0xFF, 0xE1, 0, 4, 0x67, 0x64, 0, 0x1F, 1, 0, 4, 0x68, 0xEE,
                    0x3C, 0xB0,
                ],
                width: Some(1920),
                height: Some(1080),
                sampling_frequency: None,
                channels: None,
            },
            Track {
                number: 2,
                kind: TrackKind::Audio,
                codec_id: CODEC_AAC.to_string(),
                codec_private: vec![0x11, 0x90],
                width: None,
                height: None,
                sampling_frequency: Some(48000.0),
                channels: Some(6),
            },
        )
    }

    fn walk(buffer: &[u8]) -> Vec<(String, usize, usize)> {
        let mut boxes = Vec::new();
        let mut offset = 0;
        while offset + 8 <= buffer.len() {
            let size = u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = String::from_utf8_lossy(&buffer[offset + 4..offset + 8]).to_string();
            assert!(size >= 8, "box {kind} at {offset} has size {size}");
            assert!(offset + size <= buffer.len(), "box {kind} overruns buffer");
            boxes.push((kind, offset, size));
            offset += size;
        }
        assert_eq!(offset, buffer.len(), "trailing bytes after last box");
        boxes
    }

    #[test]
    fn init_segment_declares_both_tracks() {
        let (video, audio) = sample_tracks();
        let init = write_init(1_000_000, Some(&video), Some(&audio));
        let boxes = walk(&init);
        assert_eq!(boxes[0].0, "ftyp");
        assert_eq!(boxes[1].0, "moov");

        let moov = &init[boxes[1].1 + 8..boxes[1].1 + boxes[1].2];
        let children: Vec<String> = walk(moov).into_iter().map(|entry| entry.0).collect();
        assert_eq!(children, vec!["mvhd", "trak", "trak", "mvex"]);
    }

    #[test]
    fn trun_offset_points_at_the_first_sample() {
        let (video, audio) = sample_tracks();
        let mut writer = FragmentWriter::new(1_000_000, Some(&video), Some(&audio));
        let packets = [
            Packet {
                track_number: 1,
                timestamp: 0,
                keyframe: true,
                data: vec![0xAA; 40],
            },
            Packet {
                track_number: 2,
                timestamp: 0,
                keyframe: true,
                data: vec![0xBB; 12],
            },
            Packet {
                track_number: 1,
                timestamp: 40,
                keyframe: false,
                data: vec![0xCC; 25],
            },
        ];
        for packet in &packets {
            assert!(writer.push(packet).is_none());
        }
        let fragment = writer.flush().expect("fragment");

        let boxes = walk(&fragment);
        assert_eq!(boxes[0].0, "moof");
        assert_eq!(boxes[1].0, "mdat");

        let moof_size = boxes[0].2;
        let traf_start = {
            let moof = &fragment[8..moof_size];
            let children = walk(moof);
            assert_eq!(children[0].0, "mfhd");
            assert_eq!(children[1].0, "traf");
            8 + children[1].1
        };
        let traf = &fragment[traf_start + 8..traf_start + 8 + read_size(&fragment, traf_start) - 8];
        let trun = walk(traf)
            .into_iter()
            .find(|entry| entry.0 == "trun")
            .expect("trun");
        let offset_field = traf_start + 8 + trun.1 + 8 + 4 + 4;
        let data_offset =
            u32::from_be_bytes(fragment[offset_field..offset_field + 4].try_into().unwrap())
                as usize;

        assert_eq!(data_offset, moof_size + 8);
        assert_eq!(fragment[data_offset], 0xAA);
    }

    #[test]
    fn annex_b_video_packets_are_converted_to_mp4_nal_lengths() {
        let (video, audio) = sample_tracks();
        let mut writer = FragmentWriter::new(1_000_000, Some(&video), Some(&audio));
        let packet = Packet {
            track_number: 1,
            timestamp: 0,
            keyframe: true,
            data: vec![0, 0, 0, 1, 0x67, 1, 2, 0, 0, 1, 0x68, 3],
        };
        writer.push(&packet);
        let fragment = writer.flush().expect("fragment");
        let moof_size = read_size(&fragment, 0);
        let mdat_start = moof_size;
        assert_eq!(&fragment[mdat_start + 8..mdat_start + 12], &[0, 0, 0, 3]);
        assert_eq!(&fragment[mdat_start + 12..mdat_start + 15], &[0x67, 1, 2]);
        assert_eq!(&fragment[mdat_start + 15..mdat_start + 19], &[0, 0, 0, 2]);
        assert_eq!(&fragment[mdat_start + 19..mdat_start + 21], &[0x68, 3]);
    }

    #[test]
    fn tracks_without_codec_configuration_are_not_adapted() {
        let (mut video, _) = sample_tracks();
        video.codec_private.clear();
        assert!(!supports(&video));

        let audio = Track {
            number: 2,
            kind: TrackKind::Audio,
            codec_id: CODEC_AAC.to_string(),
            codec_private: Vec::new(),
            width: None,
            height: None,
            sampling_frequency: Some(48_000.0),
            channels: Some(2),
        };
        assert!(!supports(&audio));
    }

    fn read_size(buffer: &[u8], offset: usize) -> usize {
        u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize
    }
}
