use ffmpeg_next::{
    self as ffmpeg, ChannelLayout, Rational,
    codec::{Context, Id, Parameters},
    format::{Sample, sample::Type},
    frame, software,
};

const FRAME_SAMPLES: usize = 1536;

pub fn supported(id: Id) -> bool {
    ffmpeg::decoder::find(id).is_some() && ffmpeg::encoder::find(Id::AC3).is_some()
}

pub fn eac3_available() -> bool {
    ffmpeg::encoder::find(Id::EAC3).is_some()
}

pub fn output_rate(source: u32) -> u32 {
    match source {
        11_025 | 22_050 | 44_100 | 88_200 | 176_400 | 352_800 => 44_100,
        32_000 => 32_000,
        _ => 48_000,
    }
}

pub fn output_layout(channels: i32) -> ChannelLayout {
    match channels {
        1 => ChannelLayout::MONO,
        2 => ChannelLayout::STEREO,
        _ => ChannelLayout::_5POINT1,
    }
}

pub struct Transcoder {
    parameters: Parameters,
    eac3: bool,
    rate: u32,
    decoder: ffmpeg::codec::decoder::Audio,
    encoder: ffmpeg::codec::encoder::audio::Encoder,
    resampler: Option<software::resampling::Context>,
    layout: ChannelLayout,
    pending: Vec<Vec<f32>>,
    start_ns: Option<i64>,
    frames_out: i64,
    packets_out: i64,
}

fn error(error: ffmpeg::Error) -> String {
    error.to_string()
}

impl Transcoder {
    pub fn new(parameters: &Parameters, eac3: bool) -> Result<Transcoder, String> {
        let decoder = Context::from_parameters(parameters.clone())
            .map_err(error)?
            .decoder()
            .audio()
            .map_err(error)?;
        let channels = decoder.channels() as i32;
        let layout = output_layout(channels);
        let rate = output_rate(decoder.rate());
        let codec = ffmpeg::encoder::find(if eac3 { Id::EAC3 } else { Id::AC3 }).ok_or("no AC3 encoder")?;
        let mut encoder = Context::new_with_codec(codec).encoder().audio().map_err(error)?;
        encoder.set_rate(rate as i32);
        encoder.set_channel_layout(layout);
        encoder.set_format(Sample::F32(Type::Planar));
        encoder.set_bit_rate(if layout.channels() > 2 { 640_000 } else { 192_000 });
        encoder.set_time_base(Rational::new(1, rate as i32));
        let encoder = encoder.open_as(codec).map_err(error)?;
        Ok(Transcoder {
            parameters: parameters.clone(),
            eac3,
            rate,
            decoder,
            encoder,
            resampler: None,
            layout,
            pending: vec![Vec::new(); layout.channels() as usize],
            start_ns: None,
            frames_out: 0,
            packets_out: 0,
        })
    }

    pub fn channels(&self) -> u32 {
        self.layout.channels() as u32
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn restart(&self) -> Result<Transcoder, String> {
        Transcoder::new(&self.parameters, self.eac3)
    }

    pub fn push(&mut self, packet: &ffmpeg::Packet, pts_ns: i64) -> Vec<(i64, Vec<u8>)> {
        let mut out = Vec::new();
        if self.start_ns.is_none() {
            self.start_ns = Some(pts_ns);
        }
        if self.decoder.send_packet(packet).is_err() {
            return out;
        }
        let mut decoded = frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            self.convert(&decoded);
            self.encode_ready(&mut out);
        }
        out
    }

    fn convert(&mut self, decoded: &frame::Audio) {
        if self.resampler.is_none() {
            let source = if decoded.channel_layout().is_empty() {
                ChannelLayout::default(decoded.channels() as i32)
            } else {
                decoded.channel_layout()
            };
            self.resampler = software::resampling::Context::get(
                decoded.format(),
                source,
                decoded.rate(),
                Sample::F32(Type::Planar),
                self.layout,
                self.rate,
            )
            .ok();
        }
        let Some(resampler) = self.resampler.as_mut() else {
            return;
        };
        let mut converted = frame::Audio::empty();
        if resampler.run(decoded, &mut converted).is_err() {
            return;
        }
        for (channel, pending) in self.pending.iter_mut().enumerate() {
            pending.extend_from_slice(converted.plane::<f32>(channel));
        }
    }

    fn encode_ready(&mut self, out: &mut Vec<(i64, Vec<u8>)>) {
        while self.pending.first().is_some_and(|channel| channel.len() >= FRAME_SAMPLES) {
            let mut input = frame::Audio::new(Sample::F32(Type::Planar), FRAME_SAMPLES, self.layout);
            input.set_rate(self.rate);
            input.set_pts(Some(self.frames_out * FRAME_SAMPLES as i64));
            for (channel, pending) in self.pending.iter_mut().enumerate() {
                input.plane_mut::<f32>(channel).copy_from_slice(&pending[..FRAME_SAMPLES]);
                pending.drain(..FRAME_SAMPLES);
            }
            self.frames_out += 1;
            if self.encoder.send_frame(&input).is_err() {
                return;
            }
            let start = self.start_ns.unwrap_or(0);
            let mut encoded = ffmpeg::Packet::empty();
            while self.encoder.receive_packet(&mut encoded).is_ok() {
                let samples = self.packets_out * FRAME_SAMPLES as i64;
                self.packets_out += 1;
                let pts_ns = start + (samples as i128 * 1_000_000_000 / self.rate as i128) as i64;
                if let Some(data) = encoded.data() {
                    out.push((pts_ns, data.to_vec()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_rates_stay_in_their_family() {
        assert_eq!(output_rate(44_100), 44_100);
        assert_eq!(output_rate(22_050), 44_100);
        assert_eq!(output_rate(32_000), 32_000);
        assert_eq!(output_rate(96_000), 48_000);
        assert_eq!(output_rate(24_000), 48_000);
    }
}
