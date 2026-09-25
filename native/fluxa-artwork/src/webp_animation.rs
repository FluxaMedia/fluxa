//! Structural parser for animated WebP frame packets.
//!
//! This keeps the compressed VP8/VP8L payload and animation metadata intact so
//! platform decoders can choose a hardware path without changing the existing
//! fully-composited CPU fallback.

use std::io::Cursor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimatedWebp {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub loop_count: Option<u16>,
    pub frames: Vec<AnimatedWebpFrame>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimatedWebpFrame {
    /// Canvas coordinates. ANMF stores these in multiples of two pixels.
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub duration_ms: u32,
    pub dispose_to_background: bool,
    /// `false` means source-over blend; `true` means replace the canvas area.
    pub no_blend: bool,
    /// Compressed color bitstream for VP8 hardware decoders.
    pub vp8: Option<Vec<u8>>,
    pub alpha: Option<WebpAlphaChunk>,
    /// Lossless-only frame payload; not supported by VP8/NVDEC.
    pub vp8l: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebpAlphaChunk {
    /// 0 = raw alpha samples, 1 = lossless VP8L-coded samples.
    pub compression_method: u8,
    /// WebP row filter selector (0 through 3).
    pub filter: u8,
    /// 0 = none, 1 = preprocessing level reduction.
    pub preprocessing: u8,
    /// Payload after the ALPH header byte.
    pub data: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Chunk<'a> {
    fourcc: [u8; 4],
    data: &'a [u8],
}

/// Parses an animated WebP RIFF container without decoding or composing frames.
///
/// Returns `Ok(None)` for valid non-animated WebP, and an error for malformed
/// or unsupported animation containers. Payloads are copied so the result is
/// independent of the input byte buffer's lifetime.
pub fn parse(bytes: &[u8]) -> Result<Option<AnimatedWebp>, String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err("not a WebP RIFF container".to_owned());
    }
    let riff_size = read_u32(&bytes[4..8])? as usize;
    let riff_end = 8usize
        .checked_add(riff_size)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| "WebP RIFF length exceeds input".to_owned())?;
    if riff_end < 12 {
        return Err("WebP RIFF container is truncated".to_owned());
    }

    let top_chunks = chunks(&bytes[12..riff_end])?;
    let vp8x = top_chunks.iter().find(|chunk| &chunk.fourcc == b"VP8X");
    let Some(vp8x) = vp8x else {
        return Ok(None);
    };
    if vp8x.data.len() < 10 {
        return Err("WebP VP8X header is truncated".to_owned());
    }
    if vp8x.data[1..4].iter().any(|byte| *byte != 0) {
        return Err("WebP VP8X reserved bytes are non-zero".to_owned());
    }
    let canvas_width = read_u24(&vp8x.data[4..7])? + 1;
    let canvas_height = read_u24(&vp8x.data[7..10])? + 1;
    // The animation flag is bit 1 in the VP8X feature byte.
    if vp8x.data[0] & 0x02 == 0 {
        return Ok(None);
    }

    let anim = top_chunks
        .iter()
        .find(|chunk| &chunk.fourcc == b"ANIM")
        .ok_or_else(|| "animated WebP is missing its ANIM chunk".to_owned())?;
    if anim.data.len() != 6 {
        return Err("WebP ANIM chunk must contain six bytes".to_owned());
    }
    let loop_count = Some(u16::from_le_bytes([anim.data[4], anim.data[5]]));
    let mut frames = Vec::new();
    for chunk in top_chunks.iter().filter(|chunk| &chunk.fourcc == b"ANMF") {
        let frame = parse_frame(chunk.data)?;
        let right = frame
            .x
            .checked_add(frame.width)
            .ok_or_else(|| "ANMF right edge overflow".to_owned())?;
        let bottom = frame
            .y
            .checked_add(frame.height)
            .ok_or_else(|| "ANMF bottom edge overflow".to_owned())?;
        if right > canvas_width || bottom > canvas_height {
            return Err("ANMF frame extends beyond the WebP canvas".to_owned());
        }
        frames.push(frame);
    }
    if frames.is_empty() {
        return Err("animated WebP has no ANMF frames".to_owned());
    }
    Ok(Some(AnimatedWebp {
        canvas_width,
        canvas_height,
        loop_count,
        frames,
    }))
}

fn parse_frame(data: &[u8]) -> Result<AnimatedWebpFrame, String> {
    if data.len() < 16 {
        return Err("WebP ANMF frame header is truncated".to_owned());
    }
    let x = read_u24(&data[0..3])?
        .checked_mul(2)
        .ok_or("ANMF x overflow")?;
    let y = read_u24(&data[3..6])?
        .checked_mul(2)
        .ok_or("ANMF y overflow")?;
    let width = read_u24(&data[6..9])? + 1;
    let height = read_u24(&data[9..12])? + 1;
    let duration_ms = read_u24(&data[12..15])?;
    let flags = data[15];
    if flags & !0x03 != 0 {
        return Err("WebP ANMF reserved flag bits are non-zero".to_owned());
    }
    let subchunks = chunks(&data[16..])?;
    let unique_index = |tag: &[u8; 4]| -> Result<Option<usize>, String> {
        let mut matches = subchunks
            .iter()
            .enumerate()
            .filter(|(_, chunk)| &chunk.fourcc == tag);
        let first = matches.next().map(|(index, _)| index);
        if matches.next().is_some() {
            return Err(format!(
                "ANMF frame has duplicate {} chunks",
                String::from_utf8_lossy(tag)
            ));
        }
        Ok(first)
    };
    let vp8_index = unique_index(b"VP8 ")?;
    let alpha_index = unique_index(b"ALPH")?;
    let vp8l_index = unique_index(b"VP8L")?;
    let vp8 = vp8_index.map(|index| subchunks[index].data.to_vec());
    let vp8l = vp8l_index.map(|index| subchunks[index].data.to_vec());
    if vp8.is_none() && vp8l.is_none() {
        return Err("ANMF frame has neither VP8 nor VP8L image data".to_owned());
    }
    if vp8.is_some() && vp8l.is_some() {
        return Err("ANMF frame contains conflicting VP8 and VP8L payloads".to_owned());
    }
    if alpha_index.is_some() && vp8.is_none() {
        return Err("ANMF ALPH payload requires a VP8 color payload".to_owned());
    }
    if let (Some(alpha_index), Some(vp8_index)) = (alpha_index, vp8_index)
        && alpha_index > vp8_index
    {
        return Err("ANMF ALPH chunk must precede VP8 color data".to_owned());
    }
    let alpha = alpha_index
        .map(|index| parse_alpha(subchunks[index].data))
        .transpose()?;
    Ok(AnimatedWebpFrame {
        x,
        y,
        width,
        height,
        duration_ms,
        dispose_to_background: flags & 0x01 != 0,
        no_blend: flags & 0x02 != 0,
        vp8,
        alpha,
        vp8l,
    })
}

fn parse_alpha(data: &[u8]) -> Result<WebpAlphaChunk, String> {
    let Some(&header) = data.first() else {
        return Err("ANMF ALPH payload is missing its header byte".to_owned());
    };
    if header & 0xc0 != 0 {
        return Err("WebP ALPH reserved header bits are non-zero".to_owned());
    }
    let compression_method = header & 0x03;
    if compression_method > 1 {
        return Err("unsupported WebP ALPH compression method".to_owned());
    }
    let preprocessing = (header >> 4) & 0x03;
    if preprocessing > 1 {
        return Err("unsupported WebP ALPH preprocessing method".to_owned());
    }
    Ok(WebpAlphaChunk {
        compression_method,
        filter: (header >> 2) & 0x03,
        preprocessing,
        data: data[1..].to_vec(),
    })
}

/// Decodes a WebP ALPH payload into one byte per pixel.
///
/// VP8L is used only as a small auxiliary alpha stream here; lossy VP8 color
/// remains eligible for NVDEC. Unsupported/raw alpha modes should stay on the
/// established full-frame decoder path.
pub fn decode_alpha(width: u32, height: u32, alpha: &WebpAlphaChunk) -> Result<Vec<u8>, String> {
    if width == 0
        || height == 0
        || width > super::MAX_DECODE_SIDE
        || height > super::MAX_DECODE_SIDE
    {
        return Err("WebP alpha dimensions are outside safe bounds".to_owned());
    }
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| "WebP alpha dimensions overflow".to_owned())?;
    if alpha.compression_method == 0 {
        if alpha.data.len() < pixel_count {
            return Err("raw WebP alpha payload is truncated".to_owned());
        }
        let mut plane = alpha.data[..pixel_count].to_vec();
        undo_alpha_filter(&mut plane, width as usize, height as usize, alpha.filter);
        return Ok(plane);
    }
    if alpha.compression_method != 1 {
        return Err("unsupported WebP alpha compression method".to_owned());
    }

    // An ALPH chunk carries the VP8L image stream without its normal 5-byte
    // image header. Add that header and wrap it as a standalone VP8L WebP so
    // the existing bounded decoder can perform entropy/transform decoding.
    let dimensions = ((width - 1) & 0x3fff) | (((height - 1) & 0x3fff) << 14) | (1 << 28);
    let mut vp8l = Vec::with_capacity(5 + alpha.data.len());
    vp8l.push(0x2f);
    vp8l.extend_from_slice(&dimensions.to_le_bytes());
    vp8l.extend_from_slice(&alpha.data);
    let mut webp_body = Vec::with_capacity(4 + 8 + vp8l.len() + 1);
    webp_body.extend_from_slice(b"WEBP");
    append_chunk(*b"VP8L", &vp8l, &mut webp_body)?;
    let mut webp = Vec::with_capacity(8 + webp_body.len());
    webp.extend_from_slice(b"RIFF");
    webp.extend_from_slice(
        &u32::try_from(webp_body.len())
            .map_err(|_| "WebP alpha stream is too large".to_owned())?
            .to_le_bytes(),
    );
    webp.extend_from_slice(&webp_body);

    let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(webp))
        .map_err(|error| format!("decode WebP alpha stream: {error}"))?;
    if image::ImageDecoder::dimensions(&decoder) != (width, height)
        || image::ImageDecoder::color_type(&decoder) != image::ColorType::Rgba8
    {
        return Err("WebP alpha stream decoded with unexpected dimensions or format".to_owned());
    }
    let mut rgba = vec![0; pixel_count * 4];
    image::ImageDecoder::read_image(decoder, &mut rgba)
        .map_err(|error| format!("read WebP alpha pixels: {error}"))?;
    let mut plane = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[1])
        .collect::<Vec<_>>();
    undo_alpha_filter(&mut plane, width as usize, height as usize, alpha.filter);
    Ok(plane)
}

fn undo_alpha_filter(alpha: &mut [u8], width: usize, height: usize, filter: u8) {
    if filter == 0 {
        return;
    }
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let left = if x == 0 {
                if y != 0 && matches!(filter, 1 | 3) {
                    alpha[index - width]
                } else {
                    0
                }
            } else {
                alpha[index - 1]
            };
            let above = if y == 0 {
                if x != 0 && matches!(filter, 2 | 3) {
                    alpha[index - 1]
                } else {
                    0
                }
            } else {
                alpha[index - width]
            };
            let upper_left = if x == 0 || y == 0 {
                0
            } else {
                alpha[index - width - 1]
            };
            let predictor = match filter {
                1 => left,
                2 => above,
                3 => {
                    let gradient = i16::from(left) + i16::from(above) - i16::from(upper_left);
                    gradient.clamp(0, 255) as u8
                }
                _ => 0,
            };
            alpha[index] = alpha[index].wrapping_add(predictor);
        }
    }
}

fn append_chunk(tag: [u8; 4], data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
    out.extend_from_slice(&tag);
    out.extend_from_slice(
        &u32::try_from(data.len())
            .map_err(|_| "WebP chunk is too large".to_owned())?
            .to_le_bytes(),
    );
    out.extend_from_slice(data);
    if data.len() & 1 != 0 {
        out.push(0);
    }
    Ok(())
}

fn chunks(mut data: &[u8]) -> Result<Vec<Chunk<'_>>, String> {
    let mut output = Vec::new();
    while !data.is_empty() {
        if data.len() < 8 {
            return Err("WebP chunk header is truncated".to_owned());
        }
        let fourcc = data[..4].try_into().expect("four-byte chunk tag");
        let size = read_u32(&data[4..8])? as usize;
        let end = 8usize
            .checked_add(size)
            .filter(|end| *end <= data.len())
            .ok_or_else(|| "WebP chunk length exceeds container".to_owned())?;
        output.push(Chunk {
            fourcc,
            data: &data[8..end],
        });
        let padded_end = end + (size & 1);
        if padded_end > data.len() {
            return Err("WebP chunk padding is truncated".to_owned());
        }
        data = &data[padded_end..];
    }
    Ok(output)
}

fn read_u24(bytes: &[u8]) -> Result<u32, String> {
    let bytes: [u8; 3] = bytes
        .get(..3)
        .ok_or_else(|| "WebP 24-bit field is truncated".to_owned())?
        .try_into()
        .expect("three-byte field");
    Ok(u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16))
}

fn read_u32(bytes: &[u8]) -> Result<u32, String> {
    let bytes: [u8; 4] = bytes
        .get(..4)
        .ok_or_else(|| "WebP 32-bit field is truncated".to_owned())?
        .try_into()
        .expect("four-byte field");
    Ok(u32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(tag: &[u8; 4], data: &[u8], out: &mut Vec<u8>) {
        out.extend_from_slice(tag);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        if data.len() & 1 != 0 {
            out.push(0);
        }
    }

    fn u24(value: u32) -> [u8; 3] {
        [value as u8, (value >> 8) as u8, (value >> 16) as u8]
    }

    fn sample_webp() -> Vec<u8> {
        let mut vp8x = [0u8; 10];
        vp8x[0] = 0x02;
        vp8x[4..7].copy_from_slice(&u24(319));
        vp8x[7..10].copy_from_slice(&u24(179));
        let mut anim = Vec::from([0u8; 4]);
        anim.extend_from_slice(&3u16.to_le_bytes());
        let mut frame = Vec::new();
        frame.extend_from_slice(&u24(4)); // x = 8
        frame.extend_from_slice(&u24(7)); // y = 14
        frame.extend_from_slice(&u24(99)); // width = 100
        frame.extend_from_slice(&u24(49)); // height = 50
        frame.extend_from_slice(&u24(40));
        frame.push(0x02); // no-blend, keep after display
        chunk(b"ALPH", &[1, 2, 3, 4], &mut frame);
        chunk(b"VP8 ", &[4, 5, 6, 7], &mut frame);
        let mut content = Vec::from(*b"WEBP");
        chunk(b"VP8X", &vp8x, &mut content);
        chunk(b"ANIM", &anim, &mut content);
        chunk(b"ANMF", &frame, &mut content);
        let mut webp = Vec::from(*b"RIFF");
        webp.extend_from_slice(&(content.len() as u32).to_le_bytes());
        webp.extend_from_slice(&content);
        webp
    }

    #[test]
    fn parses_frame_packets_and_composition_metadata() {
        let parsed = parse(&sample_webp()).unwrap().unwrap();
        assert_eq!((parsed.canvas_width, parsed.canvas_height), (320, 180));
        assert_eq!(parsed.loop_count, Some(3));
        assert_eq!(parsed.frames.len(), 1);
        let frame = &parsed.frames[0];
        assert_eq!(
            (frame.x, frame.y, frame.width, frame.height),
            (8, 14, 100, 50)
        );
        assert_eq!(frame.duration_ms, 40);
        assert!(!frame.dispose_to_background);
        assert!(frame.no_blend);
        assert_eq!(frame.vp8.as_deref(), Some(&[4, 5, 6, 7][..]));
        assert_eq!(
            frame.alpha,
            Some(WebpAlphaChunk {
                compression_method: 1,
                filter: 0,
                preprocessing: 0,
                data: vec![2, 3, 4],
            })
        );
        assert!(frame.vp8l.is_none());
    }

    #[test]
    fn rejects_truncated_chunk_without_panicking() {
        let mut bytes = sample_webp();
        bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse(&bytes).is_err());
    }

    #[test]
    fn rejects_frame_outside_canvas() {
        let mut bytes = sample_webp();
        let anmf = bytes
            .windows(4)
            .position(|window| window == b"ANMF")
            .unwrap();
        bytes[anmf + 8..anmf + 11].copy_from_slice(&u24(111));
        assert!(parse(&bytes).is_err());
    }

    #[test]
    fn decodes_and_unfilters_raw_horizontal_alpha() {
        let alpha = WebpAlphaChunk {
            compression_method: 0,
            filter: 1,
            preprocessing: 0,
            data: vec![10, 10, 10, 1, 1, 1],
        };
        assert_eq!(
            decode_alpha(3, 2, &alpha).unwrap(),
            vec![10, 20, 30, 11, 12, 13]
        );
    }

    #[test]
    fn decodes_and_unfilters_raw_gradient_alpha() {
        let alpha = WebpAlphaChunk {
            compression_method: 0,
            filter: 3,
            preprocessing: 0,
            data: vec![10, 10, 10, 1, 1, 1],
        };
        assert_eq!(
            decode_alpha(3, 2, &alpha).unwrap(),
            vec![10, 30, 70, 21, 42, 83]
        );
    }
}
