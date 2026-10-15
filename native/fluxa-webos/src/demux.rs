use ffmpeg_next::{self as ffmpeg, Rational, codec::Id, ffi, format, media};

use crate::annexb::{Converter, is_annexb};
use crate::load::{AudioCodec, AudioStream, VideoCodec, VideoStream};
use crate::player::{Demuxer, MediaInfo, Packet};

pub struct FfmpegDemuxer {
    input: Option<format::context::Input>,
    video: Option<usize>,
    audio: Option<usize>,
    converter: Option<Converter>,
    aac_raw: bool,
}

impl Default for FfmpegDemuxer {
    fn default() -> FfmpegDemuxer {
        FfmpegDemuxer { input: None, video: None, audio: None, converter: None, aac_raw: false }
    }
}

fn to_ns(value: i64, base: Rational) -> i64 {
    (value as i128 * base.numerator() as i128 * 1_000_000_000 / base.denominator().max(1) as i128) as i64
}

fn from_ns(ns: i64, base: Rational) -> i64 {
    (ns as i128 * base.denominator() as i128 / (base.numerator().max(1) as i128 * 1_000_000_000)) as i64
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

fn audio_codec(id: Id, profile: i32, raw: bool) -> Option<AudioCodec> {
    Some(match id {
        Id::AAC => AudioCodec::Aac { profile: profile.clamp(0, 255) as u8, raw },
        Id::AC3 => AudioCodec::Ac3,
        Id::EAC3 => AudioCodec::Eac3 { atmos: profile == ffi::AV_PROFILE_EAC3_DDP_ATMOS },
        Id::OPUS => AudioCodec::Opus,
        Id::MP3 => AudioCodec::Mp3,
        Id::DTS => AudioCodec::Dts,
        _ => return None,
    })
}

impl Demuxer for FfmpegDemuxer {
    fn open(&mut self, url: &str) -> Result<MediaInfo, String> {
        ffmpeg::init().map_err(|error| error.to_string())?;
        let input = format::input(&url).map_err(|error| error.to_string())?;

        let stream = input.streams().best(media::Type::Video).ok_or("no video stream")?;
        let parameters = stream.parameters();
        let codec = video_codec(parameters.id()).ok_or_else(|| format!("unsupported video codec {:?}", parameters.id()))?;
        let (width, height) = unsafe {
            let raw = &*parameters.as_ptr();
            (raw.width as u32, raw.height as u32)
        };
        let rate = stream.avg_frame_rate();
        let fps = (rate.numerator() > 0 && rate.denominator() > 0).then(|| (rate.numerator() as u32, rate.denominator() as u32));
        let extra = extradata(&parameters);
        self.converter = match codec {
            VideoCodec::H264 => Converter::avcc(&extra),
            VideoCodec::H265 => Converter::hvcc(&extra),
            _ => None,
        };
        self.video = Some(stream.index());
        let video = VideoStream { codec, width, height, fps, dolby_vision: None };

        let mut audio = None;
        self.audio = None;
        if let Some(stream) = input.streams().best(media::Type::Audio) {
            let parameters = stream.parameters();
            let extra = extradata(&parameters);
            let (channels, rate, profile) = unsafe {
                let raw = &*parameters.as_ptr();
                (raw.ch_layout.nb_channels as u32, raw.sample_rate as u32, raw.profile)
            };
            self.aac_raw = !extra.is_empty();
            if let Some(codec) = audio_codec(parameters.id(), profile, self.aac_raw) {
                self.audio = Some(stream.index());
                audio = Some(AudioStream { codec, channels, sample_rate: rate, extradata: extra });
            }
        }

        let duration_ns = to_ns(input.duration().max(0), Rational::new(1, ffi::AV_TIME_BASE as i32));
        self.input = Some(input);
        Ok(MediaInfo { video, audio, duration_ns, hdr: None })
    }

    fn read(&mut self) -> Option<Packet> {
        let input = self.input.as_mut()?;
        for (stream, packet) in input.packets() {
            let index = stream.index();
            let is_video = self.video.as_ref().is_some_and(|selected| *selected == index);
            let is_audio = self.audio.as_ref().is_some_and(|selected| *selected == index);
            if !is_video && !is_audio {
                continue;
            }
            let (Some(data), Some(pts)) = (packet.data(), packet.pts().or(packet.dts())) else {
                continue;
            };
            let base = stream.time_base();
            let data = match (&self.converter, is_video) {
                (Some(converter), true) if !is_annexb(data) => converter.convert(data, packet.is_key()),
                _ => data.to_vec(),
            };
            return Some(Packet { video: is_video, pts_ns: to_ns(pts, base), data });
        }
        None
    }

    fn seek(&mut self, pts_ns: i64) -> Result<(), String> {
        let input = self.input.as_mut().ok_or("not open")?;
        let target = from_ns(pts_ns, Rational::new(1, ffi::AV_TIME_BASE as i32));
        input.seek(target, ..target + 1).map_err(|error| error.to_string())
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
