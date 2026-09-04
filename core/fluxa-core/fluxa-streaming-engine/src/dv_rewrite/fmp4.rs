use super::*;

enum FMp4State {
    /// Waiting to accumulate an 8-byte ISO-BMFF box header.
    Header,
    /// Forwarding a non-mdat box's content verbatim.
    Forward {
        remaining: u64,
    },
    /// Accumulating mdat payload before NAL processing (box size is known).
    Mdat {
        buf: Vec<u8>,
        remaining: u64,
    },
    MdatSpool {
        spool: MdatSpool,
        remaining: u64,
    },
    MdatOutput {
        spool: MdatSpool,
        remaining: u64,
        header_written: bool,
    },
    /// Accumulating mdat payload that extends to EOF (box size field = 0).
    MdatEof {
        buf: Vec<u8>,
    },
    MdatEofSpool {
        spool: MdatSpool,
    },
    ForwardEof,
}

pub(crate) struct FMp4NalRewriter {
    state: FMp4State,
    header_buf: Vec<u8>,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
    spool_dir: Option<PathBuf>,
}

pub(crate) const FMP4_MDAT_RAM_LIMIT: u64 = 16 * 1024 * 1024;

struct MdatSpool {
    file: File,
    path: PathBuf,
}

fn read_mdat_spool(mut spool: MdatSpool) -> std::io::Result<Vec<u8>> {
    spool.file.seek(SeekFrom::Start(0))?;
    let mut output = Vec::new();
    spool.file.read_to_end(&mut output)?;
    Ok(output)
}

impl Drop for MdatSpool {
    fn drop(&mut self) {
        let _ = remove_file(&self.path);
    }
}

fn new_mdat_spool(spool_dir: Option<&std::path::Path>) -> std::io::Result<MdatSpool> {
    let directory = spool_dir
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory)?;
    cleanup_stale_mdat_spools(&directory);
    let path = directory.join(format!("fluxa-fmp4-{}.mdat", next_local_stream_id()));
    let file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)?;
    Ok(MdatSpool { file, path })
}

fn cleanup_stale_mdat_spools(directory: &std::path::Path) {
    static CLEANED_DIRECTORIES: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let cleaned = CLEANED_DIRECTORIES.get_or_init(|| Mutex::new(HashSet::new()));
    let Ok(mut directories) = cleaned.lock() else {
        return;
    };
    if directories.contains(directory) {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("fluxa-fmp4-") && name.ends_with(".mdat"))
            {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    directories.insert(directory.to_path_buf());
}

struct LengthDelimitedRewriteState {
    pending: Vec<u8>,
    rpu_converted: u32,
    rpu_failed: u32,
    el_dropped: u32,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
}

impl LengthDelimitedRewriteState {
    pub(crate) fn new(rpu_mode: u8, zero_level5: bool, remove_hdr10plus: bool) -> Self {
        Self {
            pending: Vec::with_capacity(65536),
            rpu_converted: 0,
            rpu_failed: 0,
            el_dropped: 0,
            rpu_mode,
            zero_level5,
            remove_hdr10plus,
        }
    }

    fn process_into(&mut self, input: &[u8], output: &mut Vec<u8>) {
        self.pending.extend_from_slice(input);
        let mut consumed = 0;
        while self.pending.len() - consumed >= 4 {
            let len = u32::from_be_bytes([
                self.pending[consumed],
                self.pending[consumed + 1],
                self.pending[consumed + 2],
                self.pending[consumed + 3],
            ]) as usize;
            let end = consumed.saturating_add(4).saturating_add(len);
            if end > self.pending.len() {
                break;
            }
            emit_length_delimited_nal(
                &self.pending[consumed..end],
                output,
                self.rpu_mode,
                self.zero_level5,
                self.remove_hdr10plus,
                &mut self.rpu_converted,
                &mut self.rpu_failed,
                &mut self.el_dropped,
            );
            consumed = end;
        }
        if consumed > 0 {
            self.pending.copy_within(consumed.., 0);
            self.pending.truncate(self.pending.len() - consumed);
        }
    }

    fn flush_into(&mut self, output: &mut Vec<u8>) {
        if !self.pending.is_empty() {
            output.extend_from_slice(&self.pending);
            self.pending.clear();
        }
    }
}

fn emit_length_delimited_nal(
    framed: &[u8],
    output: &mut Vec<u8>,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
    rpu_converted: &mut u32,
    rpu_failed: &mut u32,
    el_dropped: &mut u32,
) {
    let nal = &framed[4..];
    if nal.len() < 2 {
        output.extend_from_slice(framed);
        return;
    }
    let nal_type = (nal[0] >> 1) & 0x3F;
    let layer_id = ((nal[0] & 0x01) << 5) | (nal[1] >> 3);
    if nal_type == 62 {
        if let Some(converted) = convert_rpu_nal(nal, rpu_mode, zero_level5) {
            output.extend_from_slice(&(converted.len() as u32).to_be_bytes());
            output.extend_from_slice(&converted);
            *rpu_converted += 1;
        } else {
            output.extend_from_slice(framed);
            *rpu_failed += 1;
        }
    } else if layer_id > 0 {
        *el_dropped += 1;
    } else if !(remove_hdr10plus && nal_is_hdr10plus_sei(nal)) {
        output.extend_from_slice(framed);
    }
}

fn rewrite_mdat_spool(
    mut spool: MdatSpool,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
) -> Result<(Vec<u8>, u32, u32, u32), (std::io::Error, MdatSpool)> {
    if let Err(error) = spool.file.seek(SeekFrom::Start(0)) {
        return Err((error, spool));
    }
    let mut state = LengthDelimitedRewriteState::new(rpu_mode, zero_level5, remove_hdr10plus);
    let mut output = Vec::new();
    let mut input = [0u8; 65536];
    loop {
        let read = match spool.file.read(&mut input) {
            Ok(read) => read,
            Err(error) => return Err((error, spool)),
        };
        if read == 0 {
            break;
        }
        state.process_into(&input[..read], &mut output);
    }
    state.flush_into(&mut output);
    Ok((
        output,
        state.rpu_converted,
        state.rpu_failed,
        state.el_dropped,
    ))
}

fn rewrite_mdat_spool_to_spool(
    mut source: MdatSpool,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
    spool_dir: Option<&std::path::Path>,
) -> Result<(MdatSpool, u64, u32, u32, u32), (std::io::Error, MdatSpool)> {
    if let Err(error) = source.file.seek(SeekFrom::Start(0)) {
        return Err((error, source));
    }
    let mut target = match new_mdat_spool(spool_dir) {
        Ok(target) => target,
        Err(error) => return Err((error, source)),
    };
    let mut state = LengthDelimitedRewriteState::new(rpu_mode, zero_level5, remove_hdr10plus);
    let mut input = [0u8; 65536];
    let mut output = Vec::with_capacity(65536);
    loop {
        let read = match source.file.read(&mut input) {
            Ok(read) => read,
            Err(error) => return Err((error, source)),
        };
        if read == 0 {
            break;
        }
        output.clear();
        state.process_into(&input[..read], &mut output);
        if let Err(error) = target.file.write_all(&output) {
            return Err((error, source));
        }
    }
    output.clear();
    state.flush_into(&mut output);
    if let Err(error) = target.file.write_all(&output) {
        return Err((error, source));
    }
    let length = match target.file.stream_position() {
        Ok(length) => length,
        Err(error) => return Err((error, source)),
    };
    if let Err(error) = target.file.seek(SeekFrom::Start(0)) {
        return Err((error, source));
    }
    Ok((
        target,
        length,
        state.rpu_converted,
        state.rpu_failed,
        state.el_dropped,
    ))
}

impl FMp4NalRewriter {
    #[cfg(test)]
    pub(crate) fn new(rpu_mode: u8, zero_level5: bool, remove_hdr10plus: bool) -> Self {
        Self::with_spool_dir(rpu_mode, zero_level5, remove_hdr10plus, None)
    }

    pub(crate) fn with_spool_dir(
        rpu_mode: u8,
        zero_level5: bool,
        remove_hdr10plus: bool,
        spool_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            state: FMp4State::Header,
            header_buf: Vec::with_capacity(8),
            rpu_mode,
            zero_level5,
            remove_hdr10plus,
            spool_dir,
        }
    }

    fn drain_mdat_output(
        spool: &mut MdatSpool,
        remaining: &mut u64,
        header_written: &mut bool,
        output: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        if !*header_written {
            let size = remaining.saturating_add(8) as u32;
            output.extend_from_slice(&size.to_be_bytes());
            output.extend_from_slice(b"mdat");
            *header_written = true;
        }
        if *remaining == 0 {
            return Ok(());
        }
        let take = (*remaining).min(65536) as usize;
        let start = output.len();
        output.resize(start + take, 0);
        let read = spool.file.read(&mut output[start..])?;
        output.truncate(start + read);
        *remaining -= read as u64;
        Ok(())
    }

    pub(crate) fn process(&mut self, input: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            // Take ownership of state to avoid borrow issues in the match arms.
            let state = std::mem::replace(&mut self.state, FMp4State::Header);
            match state {
                FMp4State::Header => {
                    let needed = 8usize.saturating_sub(self.header_buf.len());
                    let take = needed.min(input.len() - pos);
                    self.header_buf.extend_from_slice(&input[pos..pos + take]);
                    pos += take;

                    if self.header_buf.len() < 8 {
                        // Stay in Header state (already set by replace above).
                        break;
                    }

                    let size_field = u32::from_be_bytes([
                        self.header_buf[0],
                        self.header_buf[1],
                        self.header_buf[2],
                        self.header_buf[3],
                    ]);
                    let is_mdat = self.header_buf[4..8] == *b"mdat";
                    let header = std::mem::take(&mut self.header_buf);

                    self.state = if is_mdat {
                        match size_field {
                            // size=0: mdat extends to EOF
                            0 => match new_mdat_spool(self.spool_dir.as_deref()) {
                                Ok(spool) => FMp4State::MdatEofSpool { spool },
                                Err(_) => FMp4State::MdatEof { buf: Vec::new() },
                            },
                            // size=1: 64-bit extended size — rare, treat as opaque forward
                            1 => {
                                out.extend_from_slice(&header);
                                FMp4State::Forward {
                                    remaining: u64::MAX,
                                }
                            }
                            n => {
                                let content = (n as u64).saturating_sub(8);
                                if content == 0 {
                                    // Empty mdat: write header unchanged, return to box parsing.
                                    out.extend_from_slice(&header);
                                    FMp4State::Header
                                } else {
                                    // Buffer the mdat payload; write corrected header after processing.
                                    if content > FMP4_MDAT_RAM_LIMIT {
                                        match new_mdat_spool(self.spool_dir.as_deref()) {
                                            Ok(spool) => FMp4State::MdatSpool {
                                                spool,
                                                remaining: content,
                                            },
                                            Err(_) => FMp4State::Mdat {
                                                buf: Vec::with_capacity(
                                                    content.min(32 * 1024 * 1024) as usize,
                                                ),
                                                remaining: content,
                                            },
                                        }
                                    } else {
                                        FMp4State::Mdat {
                                            buf: Vec::with_capacity(content as usize),
                                            remaining: content,
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        out.extend_from_slice(&header);
                        match size_field {
                            0 | 1 => FMp4State::Forward {
                                remaining: u64::MAX,
                            },
                            n => {
                                let content = (n as u64).saturating_sub(8);
                                if content == 0 {
                                    FMp4State::Header
                                } else {
                                    FMp4State::Forward { remaining: content }
                                }
                            }
                        }
                    };
                }

                FMp4State::Forward { mut remaining } => {
                    let available = (input.len() - pos) as u64;
                    let take = if remaining == u64::MAX {
                        available
                    } else {
                        available.min(remaining)
                    };
                    out.extend_from_slice(&input[pos..pos + take as usize]);
                    pos += take as usize;
                    if remaining != u64::MAX {
                        remaining -= take;
                        self.state = if remaining == 0 {
                            FMp4State::Header
                        } else {
                            FMp4State::Forward { remaining }
                        };
                    } else {
                        self.state = FMp4State::Forward {
                            remaining: u64::MAX,
                        };
                    }
                }

                FMp4State::Mdat {
                    mut buf,
                    mut remaining,
                } => {
                    let available = (input.len() - pos) as u64;
                    let take = available.min(remaining) as usize;
                    buf.extend_from_slice(&input[pos..pos + take]);
                    pos += take;
                    remaining -= take as u64;

                    if remaining == 0 {
                        let (processed, rpu_count, rpu_fail, el_dropped) =
                            rewrite_length_delimited_nals_owned(
                                buf,
                                self.rpu_mode,
                                self.zero_level5,
                                self.remove_hdr10plus,
                            );
                        stats::add(rpu_count, rpu_fail, el_dropped);
                        let new_box_size = (processed.len() + 8) as u32;
                        out.extend_from_slice(&new_box_size.to_be_bytes());
                        out.extend_from_slice(b"mdat");
                        out.extend_from_slice(&processed);
                        self.state = FMp4State::Header;
                    } else {
                        self.state = FMp4State::Mdat { buf, remaining };
                    }
                }

                FMp4State::MdatSpool {
                    mut spool,
                    mut remaining,
                } => {
                    let available = (input.len() - pos) as u64;
                    let take = available.min(remaining) as usize;
                    let start = spool.file.stream_position().unwrap_or(0);
                    if spool.file.write_all(&input[pos..pos + take]).is_err() {
                        let written = spool
                            .file
                            .stream_position()
                            .unwrap_or(start)
                            .saturating_sub(start) as usize;
                        let rest = remaining.saturating_sub(take as u64);
                        if let Ok(mut original) = read_mdat_spool(spool) {
                            original.extend_from_slice(&input[pos + written..pos + take]);
                            let size = (original.len() as u64)
                                .saturating_add(rest)
                                .saturating_add(8) as u32;
                            out.extend_from_slice(&size.to_be_bytes());
                            out.extend_from_slice(b"mdat");
                            out.extend_from_slice(&original);
                            pos += take;
                            self.state = if rest == 0 {
                                FMp4State::Header
                            } else {
                                FMp4State::Forward { remaining: rest }
                            };
                            continue;
                        }
                        return out;
                    }
                    pos += take;
                    remaining -= take as u64;
                    if remaining == 0 {
                        let result = rewrite_mdat_spool_to_spool(
                            spool,
                            self.rpu_mode,
                            self.zero_level5,
                            self.remove_hdr10plus,
                            self.spool_dir.as_deref(),
                        );
                        match result {
                            Ok((spool, length, rpu_count, rpu_fail, el_dropped)) => {
                                stats::add(rpu_count, rpu_fail, el_dropped);
                                self.state = FMp4State::MdatOutput {
                                    spool,
                                    remaining: length,
                                    header_written: false,
                                };
                            }
                            Err((_, mut source)) => {
                                let length = source.file.seek(SeekFrom::End(0)).unwrap_or(0);
                                let _ = source.file.seek(SeekFrom::Start(0));
                                self.state = FMp4State::MdatOutput {
                                    spool: source,
                                    remaining: length,
                                    header_written: false,
                                };
                            }
                        };
                    } else {
                        self.state = FMp4State::MdatSpool { spool, remaining };
                    }
                }

                FMp4State::MdatOutput {
                    mut spool,
                    mut remaining,
                    mut header_written,
                } => {
                    if Self::drain_mdat_output(
                        &mut spool,
                        &mut remaining,
                        &mut header_written,
                        &mut out,
                    )
                    .is_err()
                    {
                        self.state = FMp4State::Header;
                    } else if remaining == 0 {
                        self.state = FMp4State::Header;
                    } else {
                        self.state = FMp4State::MdatOutput {
                            spool,
                            remaining,
                            header_written,
                        };
                    }
                }

                FMp4State::MdatEof { mut buf } => {
                    buf.extend_from_slice(&input[pos..]);
                    pos = input.len();
                    self.state = FMp4State::MdatEof { buf };
                }

                FMp4State::MdatEofSpool { mut spool } => {
                    let start = spool.file.stream_position().unwrap_or(0);
                    if spool.file.write_all(&input[pos..]).is_err() {
                        let written = spool
                            .file
                            .stream_position()
                            .unwrap_or(start)
                            .saturating_sub(start) as usize;
                        if let Ok(mut original) = read_mdat_spool(spool) {
                            original.extend_from_slice(&input[pos + written..]);
                            out.extend_from_slice(&[0, 0, 0, 0]);
                            out.extend_from_slice(b"mdat");
                            out.extend_from_slice(&original);
                            pos = input.len();
                            self.state = FMp4State::ForwardEof;
                            continue;
                        }
                        return out;
                    }
                    pos = input.len();
                    self.state = FMp4State::MdatEofSpool { spool };
                }

                FMp4State::ForwardEof => {
                    out.extend_from_slice(&input[pos..]);
                    pos = input.len();
                    self.state = FMp4State::ForwardEof;
                }
            }
        }

        out
    }

    pub(crate) fn flush(self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.state {
            FMp4State::MdatEof { buf } => {
                let (processed, rpu_count, rpu_fail, el_dropped) =
                    rewrite_length_delimited_nals_owned(
                        buf,
                        self.rpu_mode,
                        self.zero_level5,
                        self.remove_hdr10plus,
                    );
                stats::add(rpu_count, rpu_fail, el_dropped);
                // Preserve size=0 (EOF-scoped) semantics in the output box.
                out.extend_from_slice(&[0, 0, 0, 0]);
                out.extend_from_slice(b"mdat");
                out.extend_from_slice(&processed);
            }
            FMp4State::MdatEofSpool { spool } => {
                match rewrite_mdat_spool(
                    spool,
                    self.rpu_mode,
                    self.zero_level5,
                    self.remove_hdr10plus,
                ) {
                    Ok((processed, rpu_count, rpu_fail, el_dropped)) => {
                        stats::add(rpu_count, rpu_fail, el_dropped);
                        out.extend_from_slice(&[0, 0, 0, 0]);
                        out.extend_from_slice(b"mdat");
                        out.extend_from_slice(&processed);
                    }
                    Err((_, spool)) => {
                        if let Ok(original) = read_mdat_spool(spool) {
                            out.extend_from_slice(&[0, 0, 0, 0]);
                            out.extend_from_slice(b"mdat");
                            out.extend_from_slice(&original);
                        }
                    }
                }
            }
            FMp4State::MdatOutput {
                mut spool,
                mut remaining,
                mut header_written,
            } => {
                while remaining > 0 || !header_written {
                    if Self::drain_mdat_output(
                        &mut spool,
                        &mut remaining,
                        &mut header_written,
                        &mut out,
                    )
                    .is_err()
                    {
                        break;
                    }
                }
            }
            FMp4State::Header if !self.header_buf.is_empty() => {
                // Incomplete box header at EOF: forward the partial bytes as-is.
                out.extend_from_slice(&self.header_buf);
            }
            _ => {}
        }
        out
    }

    pub(crate) fn flush_streaming(&mut self) -> Vec<u8> {
        let state = std::mem::replace(&mut self.state, FMp4State::Header);
        let mut out = Vec::with_capacity(65544);
        match state {
            FMp4State::MdatOutput {
                mut spool,
                mut remaining,
                mut header_written,
            } => {
                let ok = Self::drain_mdat_output(
                    &mut spool,
                    &mut remaining,
                    &mut header_written,
                    &mut out,
                )
                .is_ok();
                if ok && remaining > 0 {
                    self.state = FMp4State::MdatOutput {
                        spool,
                        remaining,
                        header_written,
                    };
                }
            }
            FMp4State::MdatEof { buf } => {
                let (processed, rpu_count, rpu_fail, el_dropped) =
                    rewrite_length_delimited_nals_owned(
                        buf,
                        self.rpu_mode,
                        self.zero_level5,
                        self.remove_hdr10plus,
                    );
                stats::add(rpu_count, rpu_fail, el_dropped);
                out.extend_from_slice(&[0, 0, 0, 0]);
                out.extend_from_slice(b"mdat");
                out.extend_from_slice(&processed);
            }
            FMp4State::MdatEofSpool { spool } => {
                match rewrite_mdat_spool_to_spool(
                    spool,
                    self.rpu_mode,
                    self.zero_level5,
                    self.remove_hdr10plus,
                    self.spool_dir.as_deref(),
                ) {
                    Ok((spool, length, rpu_count, rpu_fail, el_dropped)) => {
                        stats::add(rpu_count, rpu_fail, el_dropped);
                        self.state = FMp4State::MdatOutput {
                            spool,
                            remaining: length,
                            header_written: false,
                        };
                        return self.flush_streaming();
                    }
                    Err((_, mut source)) => {
                        let length = source.file.seek(SeekFrom::End(0)).unwrap_or(0);
                        let _ = source.file.seek(SeekFrom::Start(0));
                        self.state = FMp4State::MdatOutput {
                            spool: source,
                            remaining: length,
                            header_written: false,
                        };
                        return self.flush_streaming();
                    }
                }
            }
            FMp4State::Header if !self.header_buf.is_empty() => {
                out.extend_from_slice(&self.header_buf);
                self.header_buf.clear();
            }
            other => self.state = other,
        }
        out
    }
}

/// Scan a contiguous slice of length-delimited (4-byte BE prefix) HEVC NAL units
/// and rewrite DV7 RPU/EL NALs for DV8.1 single-layer output.
///
/// Returns `(rewritten_payload, rpu_converted_count, el_dropped_count)`.
/// Returns `(rewritten_payload, rpu_converted, rpu_failed, el_dropped)`.
pub(crate) fn rewrite_length_delimited_nals(
    data: &[u8],
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
) -> (Vec<u8>, u32, u32, u32) {
    let mut out = Vec::with_capacity(data.len());
    let mut rpu_converted = 0u32;
    let mut rpu_failed = 0u32;
    let mut el_dropped = 0u32;
    let mut i = 0;

    while i + 4 <= data.len() {
        let nal_len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        let payload_end = i + 4 + nal_len;

        if payload_end > data.len() {
            // Truncated NAL at end of mdat — copy remainder unchanged.
            out.extend_from_slice(&data[i..]);
            break;
        }

        let nal = &data[i + 4..payload_end];
        if nal.len() >= 2 {
            let nal_type = (nal[0] >> 1) & 0x3F;
            // HEVC NAL header: nuh_layer_id lives in bits [8:3] across both header bytes.
            let layer_id = ((nal[0] & 0x01) << 5) | (nal[1] >> 3);

            if nal_type == 62 {
                // UNSPEC62 = DV RPU NAL — convert to target profile.
                match convert_rpu_nal(nal, rpu_mode, zero_level5) {
                    Some(converted) => {
                        out.extend_from_slice(&(converted.len() as u32).to_be_bytes());
                        out.extend_from_slice(&converted);
                        rpu_converted += 1;
                    }
                    None => {
                        // Conversion failed: keep original NAL unchanged.
                        out.extend_from_slice(&data[i..payload_end]);
                        rpu_failed += 1;
                    }
                }
            } else if layer_id > 0 {
                // Enhancement layer NAL — not needed for single-layer DV8.1.
                el_dropped += 1;
            } else if remove_hdr10plus && nal_is_hdr10plus_sei(nal) {
                // Single-pass: strip HDR10+ SEI alongside RPU processing.
            } else {
                out.extend_from_slice(&data[i..payload_end]);
            }
        } else {
            out.extend_from_slice(&data[i..payload_end]);
        }

        i = payload_end;
    }

    (out, rpu_converted, rpu_failed, el_dropped)
}

fn rewrite_length_delimited_nals_owned(
    data: Vec<u8>,
    rpu_mode: u8,
    zero_level5: bool,
    remove_hdr10plus: bool,
) -> (Vec<u8>, u32, u32, u32) {
    if !has_length_delimited_rewrite_target(&data, remove_hdr10plus) {
        return (data, 0, 0, 0);
    }
    rewrite_length_delimited_nals(&data, rpu_mode, zero_level5, remove_hdr10plus)
}

pub(crate) fn has_length_delimited_rewrite_target(data: &[u8], remove_hdr10plus: bool) -> bool {
    let mut i = 0;
    while i + 4 <= data.len() {
        let len = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        let end = i + 4 + len;
        if end > data.len() {
            return false;
        }
        let nal = &data[i + 4..end];
        if nal.len() >= 2 {
            let nal_type = (nal[0] >> 1) & 0x3F;
            let layer_id = ((nal[0] & 0x01) << 5) | (nal[1] >> 3);
            if nal_type == 62 || layer_id > 0 || (remove_hdr10plus && nal_is_hdr10plus_sei(nal)) {
                return true;
            }
        }
        i = end;
    }
    false
}
