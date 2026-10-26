use std::collections::VecDeque;

use ffmpeg_next::{self as ffmpeg, Rational, codec::Id, ffi, format, media};

use crate::annexb::{Converter, is_annexb};
use crate::capabilities::Capabilities;
use crate::diagnostics::note;
use crate::dovi;
use crate::hdr::{HdrInfo, HdrKind, Mastering, kind_for_transfer};
use crate::hdr10plus;
use crate::load::{AudioCodec, AudioStream, DolbyVision, VideoCodec, VideoStream};
use crate::player::{Demuxer, MediaInfo, Packet, TrackInfo};
use crate::subtitle::{self, SubtitleFormat};
use crate::transcode::{self, Transcoder};

pub struct FfmpegDemuxer {
    input: Option<format::context::Input>,
    video: Option<usize>,
    audio: Option<usize>,
    subtitle: Option<(usize, SubtitleFormat)>,
    converter: Option<Converter>,
    aac_raw: bool,
    dovi_mode: Option<dovi::Mode>,
    hevc: bool,
    capabilities: Capabilities,
    dovi_stats: dovi::Stats,
    transcoder: Option<Transcoder>,
    queue: VecDeque<Packet>,
}

impl FfmpegDemuxer {
    pub fn with_capabilities(capabilities: Capabilities) -> FfmpegDemuxer {
        FfmpegDemuxer {
            capabilities,
            ..FfmpegDemuxer::default()
        }
    }
}

impl Default for FfmpegDemuxer {
    fn default() -> FfmpegDemuxer {
        FfmpegDemuxer {
            input: None,
            video: None,
            audio: None,
            subtitle: None,
            converter: None,
            aac_raw: false,
            dovi_mode: None,
            hevc: false,
            capabilities: Capabilities::default(),
            dovi_stats: dovi::Stats::default(),
            transcoder: None,
            queue: VecDeque::new(),
        }
    }
}

fn to_ns(value: i64, base: Rational) -> i64 {
    (value as i128 * base.numerator() as i128 * 1_000_000_000 / base.denominator().max(1) as i128)
        as i64
}

fn from_ns(ns: i64, base: Rational) -> i64 {
    (ns as i128 * base.denominator() as i128 / (base.numerator().max(1) as i128 * 1_000_000_000))
        as i64
}

fn extradata(parameters: &ffmpeg::codec::Parameters) -> Vec<u8> {
    unsafe {
        let raw = &*parameters.as_ptr();
        if raw.extradata.is_null() || raw.extradata_size <= 0 {
            return Vec::new();
        }
        std::slice::from_raw_parts(raw.extradata, raw.extradata_size as usize).to_vec()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DoviRecord {
    version_major: u8,
    version_minor: u8,
    dv_profile: u8,
    dv_level: u8,
    rpu_present_flag: u8,
    el_present_flag: u8,
    bl_present_flag: u8,
    bl_signal_compatibility_id: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MasteringRecord {
    display_primaries: [[ffi::AVRational; 2]; 3],
    white_point: [ffi::AVRational; 2],
    min_luminance: ffi::AVRational,
    max_luminance: ffi::AVRational,
    has_primaries: i32,
    has_luminance: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
struct LightRecord {
    MaxCLL: u32,
    MaxFALL: u32,
}

fn rational(value: ffi::AVRational) -> f64 {
    if value.den == 0 {
        0.0
    } else {
        value.num as f64 / value.den as f64
    }
}

fn side_data<'a>(
    parameters: &'a ffmpeg::codec::Parameters,
    kind: ffi::AVPacketSideDataType,
) -> Option<&'a [u8]> {
    unsafe {
        let raw = &*parameters.as_ptr();
        if raw.coded_side_data.is_null() || raw.nb_coded_side_data <= 0 {
            return None;
        }
        std::slice::from_raw_parts(raw.coded_side_data, raw.nb_coded_side_data as usize)
            .iter()
            .find(|entry| entry.type_ == kind && !entry.data.is_null())
            .map(|entry| std::slice::from_raw_parts(entry.data as *const u8, entry.size))
    }
}

fn read<T: Copy>(bytes: &[u8]) -> Option<T> {
    (bytes.len() >= std::mem::size_of::<T>())
        .then(|| unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const T) })
}

fn dolby_vision(parameters: &ffmpeg::codec::Parameters) -> Option<(DolbyVision, u8)> {
    let record: DoviRecord = read(side_data(
        parameters,
        ffi::AVPacketSideDataType::AV_PKT_DATA_DOVI_CONF,
    )?)?;
    Some((
        DolbyVision {
            profile: record.dv_profile,
            dual_layer: record.el_present_flag != 0,
        },
        record.bl_signal_compatibility_id,
    ))
}

fn base_layer_hdr(compat: u8) -> Option<HdrInfo> {
    let (kind, transfer) = match compat {
        1 | 6 => (HdrKind::Hdr10, 16),
        4 => (HdrKind::Hlg, 18),
        _ => return None,
    };
    Some(HdrInfo {
        kind,
        mastering: None,
        max_cll: None,
        max_fall: None,
        transfer,
        primaries: 9,
        matrix: 9,
        full_range: false,
    })
}

fn hdr_info(parameters: &ffmpeg::codec::Parameters) -> Option<HdrInfo> {
    let (transfer, primaries, matrix, full_range) = unsafe {
        let raw = &*parameters.as_ptr();
        (
            raw.color_trc as u32,
            raw.color_primaries as u32,
            raw.color_space as u32,
            raw.color_range == ffi::AVColorRange::AVCOL_RANGE_JPEG,
        )
    };
    let kind = kind_for_transfer(transfer)?;
    let mastering = side_data(
        parameters,
        ffi::AVPacketSideDataType::AV_PKT_DATA_MASTERING_DISPLAY_METADATA,
    )
    .and_then(read::<MasteringRecord>)
    .map(|meta| Mastering {
        primaries: (meta.has_primaries != 0).then(|| {
            let point = |index: usize| {
                (
                    rational(meta.display_primaries[index][0]),
                    rational(meta.display_primaries[index][1]),
                )
            };
            [point(0), point(1), point(2)]
        }),
        white_point: (meta.has_primaries != 0)
            .then(|| (rational(meta.white_point[0]), rational(meta.white_point[1]))),
        luminance: (meta.has_luminance != 0)
            .then(|| (rational(meta.min_luminance), rational(meta.max_luminance))),
    })
    .unwrap_or(Mastering {
        primaries: None,
        white_point: None,
        luminance: None,
    });
    let light = side_data(
        parameters,
        ffi::AVPacketSideDataType::AV_PKT_DATA_CONTENT_LIGHT_LEVEL,
    )
    .and_then(read::<LightRecord>);
    Some(HdrInfo {
        kind,
        mastering: Some(mastering),
        max_cll: light.map(|light| light.MaxCLL),
        max_fall: light.map(|light| light.MaxFALL),
        transfer,
        primaries,
        matrix,
        full_range,
    })
}

fn video_codec(id: Id) -> Option<VideoCodec> {
    Some(match id {
        Id::H264 => VideoCodec::H264,
        Id::HEVC => VideoCodec::H265,
        Id::VP8 => VideoCodec::Vp8,
        Id::VP9 => VideoCodec::Vp9,
        Id::AV1 => VideoCodec::Av1,
        _ => return None,
    })
}

fn audio_codec(id: Id, profile: i32, raw: bool, dts: bool) -> Option<AudioCodec> {
    Some(match id {
        Id::AAC => AudioCodec::Aac {
            profile: profile.clamp(0, 255) as u8,
            raw,
        },
        Id::AC3 => AudioCodec::Ac3,
        Id::EAC3 => AudioCodec::Eac3 {
            atmos: profile == ffi::AV_PROFILE_EAC3_DDP_ATMOS,
        },
        Id::OPUS => AudioCodec::Opus,
        Id::MP3 => AudioCodec::Mp3,
        Id::DTS if dts => match profile {
            ffi::AV_PROFILE_DTS_ES => AudioCodec::DtsEs,
            ffi::AV_PROFILE_DTS_HD_MA_X | ffi::AV_PROFILE_DTS_HD_MA_X_IMAX => AudioCodec::DtsX,
            _ => AudioCodec::Dts,
        },
        _ => return None,
    })
}

fn eac3_output(capabilities: &Capabilities) -> bool {
    capabilities.eac3_out && transcode::eac3_available()
}

fn subtitle_format(id: Id) -> Option<SubtitleFormat> {
    Some(match id {
        Id::SUBRIP | Id::TEXT | Id::WEBVTT => SubtitleFormat::Text,
        Id::ASS | Id::SSA => SubtitleFormat::Ass,
        Id::MOV_TEXT => SubtitleFormat::MovText,
        _ => return None,
    })
}

fn audio_stream(
    stream: &format::stream::Stream,
    capabilities: &Capabilities,
) -> Option<(AudioStream, bool, bool)> {
    let parameters = stream.parameters();
    let extra = extradata(&parameters);
    let (channels, rate, profile) = unsafe {
        let raw = &*parameters.as_ptr();
        (
            raw.ch_layout.nb_channels as u32,
            raw.sample_rate as u32,
            raw.profile,
        )
    };
    let raw = !extra.is_empty();
    let Some(codec) = audio_codec(
        parameters.id(),
        profile,
        raw,
        capabilities.dts == Some(true),
    ) else {
        if !transcode::supported(parameters.id()) {
            return None;
        }
        let channels = transcode::output_layout(channels as i32).channels() as u32;
        let codec = if eac3_output(capabilities) {
            AudioCodec::Eac3 { atmos: false }
        } else {
            AudioCodec::Ac3
        };
        let stream = AudioStream {
            codec,
            channels,
            sample_rate: transcode::output_rate(rate),
            extradata: Vec::new(),
        };
        return Some((stream, false, true));
    };
    Some((
        AudioStream {
            codec,
            channels,
            sample_rate: rate,
            extradata: extra,
        },
        raw,
        false,
    ))
}

fn track_info(stream: &format::stream::Stream, subtitle: bool) -> TrackInfo {
    let metadata = stream.metadata();
    let text = |key: &str| {
        metadata
            .get(key)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let prefix = if subtitle { 's' } else { 'a' };
    TrackInfo {
        id: format!("{prefix}{}", stream.index()),
        subtitle,
        language: text("language"),
        title: text("title"),
    }
}

fn stream_index(id: &str) -> Option<usize> {
    id.get(1..)?.parse().ok()
}

impl Demuxer for FfmpegDemuxer {
    fn open(&mut self, url: &str) -> Result<MediaInfo, String> {
        ffmpeg::init().map_err(|error| error.to_string())?;
        let mut options = ffmpeg::Dictionary::new();
        options.set("rw_timeout", "15000000");
        options.set("reconnect", "1");
        options.set("reconnect_streamed", "1");
        options.set("reconnect_delay_max", "5");
        options.set("multiple_requests", "1");
        let input =
            format::input_with_dictionary(&url, options).map_err(|error| error.to_string())?;

        let stream = input
            .streams()
            .best(media::Type::Video)
            .ok_or("no video stream")?;
        let parameters = stream.parameters();
        let codec = video_codec(parameters.id())
            .ok_or_else(|| format!("unsupported video codec {:?}", parameters.id()))?;
        let (width, height) = unsafe {
            let raw = &*parameters.as_ptr();
            (raw.width as u32, raw.height as u32)
        };
        let rate = stream.avg_frame_rate();
        let fps = (rate.numerator() > 0 && rate.denominator() > 0)
            .then(|| (rate.numerator() as u32, rate.denominator() as u32));
        if codec == VideoCodec::H264
            && unsafe { (*parameters.as_ptr()).profile } == ffi::AV_PROFILE_H264_HIGH_10
        {
            return Err("10-bit H.264 cannot be decoded by this TV".into());
        }
        let extra = extradata(&parameters);
        self.converter = match codec {
            VideoCodec::H264 => Converter::avcc(&extra),
            VideoCodec::H265 => Converter::hvcc(&extra),
            _ => None,
        };
        self.video = Some(stream.index());
        self.hevc = codec == VideoCodec::H265;
        let mut dolby = None;
        let mut hdr = hdr_info(&parameters);
        self.dovi_mode = None;
        self.dovi_stats = dovi::Stats::default();
        if let Some((dv, compat)) = dolby_vision(&parameters).filter(|_| codec == VideoCodec::H265)
        {
            match dovi::plan(
                dv.profile,
                compat,
                &self.capabilities,
                dovi::forced_from_env(),
            ) {
                dovi::Plan::Pass => dolby = Some(dv),
                dovi::Plan::Rewrite(dovi::Mode::To81) => {
                    self.dovi_mode = Some(dovi::Mode::To81);
                    dolby = Some(DolbyVision {
                        profile: 8,
                        dual_layer: false,
                    });
                }
                dovi::Plan::Rewrite(dovi::Mode::Strip) => {
                    self.dovi_mode = Some(dovi::Mode::Strip);
                    if hdr.is_none() {
                        hdr = base_layer_hdr(compat);
                    }
                }
                dovi::Plan::Unsupported(reason) => return Err(reason.to_owned()),
            }
        }
        if self.capabilities.hdr == Some(false) {
            hdr = None;
        }
        let video = VideoStream {
            codec,
            width,
            height,
            fps,
            dolby_vision: dolby,
        };

        let mut audio = None;
        let mut audio_id = None;
        let mut tracks = Vec::new();
        self.audio = None;
        self.subtitle = None;
        self.queue.clear();
        let best = input
            .streams()
            .best(media::Type::Audio)
            .map(|stream| stream.index());
        for stream in input.streams() {
            match stream.parameters().medium() {
                media::Type::Audio => {
                    let Some((parsed, raw, convert)) = audio_stream(&stream, &self.capabilities)
                    else {
                        continue;
                    };
                    tracks.push(track_info(&stream, false));
                    if Some(stream.index()) == best {
                        self.transcoder = None;
                        if convert {
                            match Transcoder::new(
                                &stream.parameters(),
                                eac3_output(&self.capabilities),
                            ) {
                                Ok(transcoder) => self.transcoder = Some(transcoder),
                                Err(_) => continue,
                            }
                        }
                        self.aac_raw = raw;
                        self.audio = Some(stream.index());
                        audio_id = Some(format!("a{}", stream.index()));
                        audio = Some(parsed);
                    }
                }
                media::Type::Subtitle if subtitle_format(stream.parameters().id()).is_some() => {
                    tracks.push(track_info(&stream, true));
                }
                _ => {}
            }
        }

        let duration_ns = to_ns(
            input.duration().max(0),
            Rational::new(1, ffi::AV_TIME_BASE as i32),
        );
        self.input = Some(input);
        Ok(MediaInfo {
            video,
            audio,
            duration_ns,
            hdr,
            tracks,
            audio_id,
        })
    }

    fn read(&mut self) -> Option<Packet> {
        if let Some(packet) = self.queue.pop_front() {
            return Some(packet);
        }
        let input = self.input.as_mut()?;
        for (stream, packet) in input.packets() {
            let index = stream.index();
            let is_video = self
                .video
                .as_ref()
                .is_some_and(|selected| *selected == index);
            let is_audio = self
                .audio
                .as_ref()
                .is_some_and(|selected| *selected == index);
            if let Some((selected, format)) =
                self.subtitle.filter(|(selected, _)| *selected == index)
            {
                let base = stream.time_base();
                let Some(pts) = packet.pts().or(packet.dts()) else {
                    continue;
                };
                let start = to_ns(pts, base);
                let cue = packet.data().and_then(|data| {
                    subtitle::cue(format, data, start, to_ns(packet.duration(), base))
                });
                if let Some(cue) = cue {
                    return Some(Packet {
                        video: false,
                        pts_ns: start,
                        data: Vec::new(),
                        cue: Some(cue),
                    });
                }
                let _ = selected;
                continue;
            }
            if !is_video && !is_audio {
                continue;
            }
            let (Some(data), Some(pts)) = (packet.data(), packet.pts().or(packet.dts())) else {
                continue;
            };
            let base = stream.time_base();
            if to_ns(pts, base) < 0 {
                continue;
            }
            if is_audio {
                if let Some(transcoder) = self.transcoder.as_mut() {
                    for (pts_ns, data) in transcoder.push(&packet, to_ns(pts, base)) {
                        self.queue.push_back(Packet {
                            video: false,
                            pts_ns,
                            data,
                            cue: None,
                        });
                    }
                    match self.queue.pop_front() {
                        Some(packet) => return Some(packet),
                        None => continue,
                    }
                }
            }
            let data = match (self.converter.as_mut(), is_video) {
                (Some(converter), true) if !is_annexb(data) => match converter.convert(data) {
                    Some(converted) => converted,
                    None => continue,
                },
                _ => data.to_vec(),
            };
            let data = match (is_video && self.hevc, hdr10plus::strip(&data)) {
                (true, Some(stripped)) => stripped,
                _ => data,
            };
            let data = if let (true, Some(mode)) = (is_video, self.dovi_mode) {
                let before = self.dovi_stats;
                let data = dovi::rewrite(&data, mode, &mut self.dovi_stats);
                if self.dovi_stats != before {
                    note("dovi.converted", self.dovi_stats.converted.to_string());
                    note("dovi.dropped", self.dovi_stats.dropped.to_string());
                }
                data
            } else {
                data
            };
            return Some(Packet {
                video: is_video,
                pts_ns: to_ns(pts, base),
                data,
                cue: None,
            });
        }
        None
    }

    fn select_audio(&mut self, id: &str) -> Result<AudioStream, String> {
        let input = self.input.as_ref().ok_or("not open")?;
        let index = stream_index(id).ok_or("bad track id")?;
        let stream = input.stream(index).ok_or("no such track")?;
        let (parsed, raw, convert) =
            audio_stream(&stream, &self.capabilities).ok_or("unsupported audio codec")?;
        self.transcoder = if convert {
            Some(Transcoder::new(
                &stream.parameters(),
                eac3_output(&self.capabilities),
            )?)
        } else {
            None
        };
        self.queue.clear();
        self.aac_raw = raw;
        self.audio = Some(index);
        Ok(parsed)
    }

    fn select_subtitle(&mut self, id: Option<&str>) {
        self.subtitle = id.and_then(stream_index).and_then(|index| {
            let format = subtitle_format(self.input.as_ref()?.stream(index)?.parameters().id())?;
            Some((index, format))
        });
    }

    fn seek(&mut self, pts_ns: i64) -> Result<(), String> {
        let input = self.input.as_mut().ok_or("not open")?;
        let target = from_ns(pts_ns, Rational::new(1, ffi::AV_TIME_BASE as i32));
        input
            .seek(target, ..target + 1)
            .map_err(|error| error.to_string())?;
        self.queue.clear();
        if let Some(converter) = self.converter.as_mut() {
            converter.reset();
        }
        if let Some(transcoder) = &self.transcoder {
            self.transcoder = Some(transcoder.restart()?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nanosecond_conversion_round_trips_millisecond_time_base() {
        let base = Rational::new(1, 1000);
        assert_eq!(to_ns(1500, base), 1_500_000_000);
        assert_eq!(from_ns(1_500_000_000, base), 1500);
    }
}
