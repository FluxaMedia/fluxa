pub(super) fn mangle_fourcc(data: &mut [u8]) -> usize {
    let limit = data.len().saturating_sub(3);
    let mut index = 0;
    let mut count = 0;
    while index < limit {
        let value = &data[index..index + 4];
        if value == b"dvcC" || value == b"dvvC" || value == b"dvhe" || value == b"dvh1" {
            data[index..index + 4].copy_from_slice(b"XXXX");
            index += 4;
            count += 1;
        } else {
            index += 1;
        }
    }
    count
}

pub(crate) fn apply_patch_at_offset(
    data: &mut [u8],
    file_offset: u64,
    scan_window: usize,
) -> usize {
    if file_offset >= scan_window as u64 {
        return 0;
    }
    let patch_len = ((scan_window as u64 - file_offset) as usize).min(data.len());
    mangle_fourcc(&mut data[..patch_len])
}

pub(crate) fn parse_content_range_start(header: &str) -> Option<u64> {
    let value = header.strip_prefix("bytes ")?.trim();
    let (range, _) = value.split_once('/')?;
    let (start, _) = range.split_once('-')?;
    start.trim().parse().ok()
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ContainerInfo {
    pub(super) profile: u8,
    pub(super) compat_id: u8,
}

impl ContainerInfo {
    pub(super) fn not_has_hdr10_fallback(self) -> bool {
        self.profile == 4
            || (self.profile == 5 && self.compat_id != 1)
            || (self.profile == 10 && matches!(self.compat_id, 0 | 2 | 3))
    }
}

pub(super) fn scan_info(data: &[u8]) -> Option<ContainerInfo> {
    for index in 0..data.len().saturating_sub(8) {
        if data[index..index + 4] == *b"dvcC" {
            let payload = &data[index + 4..];
            if payload.len() >= 5 {
                return Some(ContainerInfo {
                    profile: (payload[2] >> 1) & 0x7F,
                    compat_id: (payload[4] >> 4) & 0x0F,
                });
            }
        }
    }
    None
}
