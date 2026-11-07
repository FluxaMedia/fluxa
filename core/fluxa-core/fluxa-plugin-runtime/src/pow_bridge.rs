use std::sync::atomic::{AtomicU32, Ordering};

const SCRATCH: usize = 512;
const MASK: usize = SCRATCH - 1;
const ROUNDS: usize = 2;
const LIMIT: u32 = 1 << 24;
const MIX_LO: u32 = 2_654_435_761;
const MIX_HI: u32 = 2_246_822_519;
const NO_SOLUTION: u32 = u32::MAX;

type State = [u32; 4];

fn quarter(s: &mut State) {
    s[0] = s[0].wrapping_add(s[1]);
    s[3] = (s[3] ^ s[0]).rotate_left(16);
    s[2] = s[2].wrapping_add(s[3]);
    s[1] = (s[1] ^ s[2]).rotate_left(12);
    s[0] = s[0].wrapping_add(s[1]);
    s[3] = (s[3] ^ s[0]).rotate_left(8);
    s[2] = s[2].wrapping_add(s[3]);
    s[1] = (s[1] ^ s[2]).rotate_left(7);
}

fn absorb(state: &mut State, bytes: &[u8]) {
    for &byte in bytes {
        state[0] = state[0].wrapping_add(byte as u32).rotate_left(7);
        quarter(state);
    }
}

fn leading_zero_bits(words: &[u32; 8], needed: u32) -> bool {
    let mut zeros = 0;
    for &word in words {
        if word == 0 {
            zeros += 32;
            continue;
        }
        zeros += word.leading_zeros();
        break;
    }
    zeros >= needed
}

fn meets(prefix: &State, tail: &[u8], needed: u32, scratch: &mut [u32; SCRATCH]) -> bool {
    let mut state = *prefix;
    absorb(&mut state, tail);
    for _ in 0..8 {
        quarter(&mut state);
    }
    for slot in scratch.iter_mut() {
        quarter(&mut state);
        *slot = state[0] ^ state[2];
    }
    for _ in 0..ROUNDS {
        for i in 0..SCRATCH {
            let cur = scratch[i];
            let mut mixed = cur
                .wrapping_add(scratch[(cur as usize) & MASK])
                .rotate_left(13);
            mixed ^= scratch[(i + 1) & MASK].wrapping_mul(MIX_LO);
            scratch[i] = mixed;
            state[0] ^= mixed;
            quarter(&mut state);
        }
    }
    let span = SCRATCH / 8;
    let mut words = [0u32; 8];
    for (i, word) in words.iter_mut().enumerate() {
        quarter(&mut state);
        let mut acc = state[0];
        for &d in &scratch[i * span..(i + 1) * span] {
            acc = acc.wrapping_add(d).rotate_left(5) ^ d.wrapping_mul(MIX_HI);
        }
        *word = acc ^ state[2];
    }
    leading_zero_bits(&words, needed)
}

fn write_digits(buf: &mut Vec<u8>, base: usize, mut value: u32) {
    buf.truncate(base);
    let start = buf.len();
    loop {
        buf.push(b'0' + (value % 10) as u8);
        value /= 10;
        if value == 0 {
            break;
        }
    }
    buf[start..].reverse();
}

pub fn solve(nonce: &str, difficulty: u32) -> Option<u32> {
    let mut prefix_state: State = [1_779_033_703, 3_144_134_277, 1_013_904_242, 2_773_480_762];
    let mut head = nonce.as_bytes().to_vec();
    head.push(b':');
    absorb(&mut prefix_state, &head);

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8) as u32;
    let best = AtomicU32::new(NO_SOLUTION);
    let prefix_ref = &prefix_state;
    let best_ref = &best;

    std::thread::scope(|scope| {
        for worker in 0..threads {
            scope.spawn(move || {
                let mut scratch = [0u32; SCRATCH];
                let mut tail = Vec::with_capacity(10);
                let mut candidate = worker;
                while candidate < LIMIT && candidate < best_ref.load(Ordering::Relaxed) {
                    write_digits(&mut tail, 0, candidate);
                    if meets(prefix_ref, &tail, difficulty, &mut scratch) {
                        best_ref.fetch_min(candidate, Ordering::Relaxed);
                        break;
                    }
                    candidate += threads;
                }
            });
        }
    });

    match best.load(Ordering::Relaxed) {
        NO_SOLUTION => None,
        found => Some(found),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONCE: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn matches_js_solver() {
        assert_eq!(solve(NONCE, 10), Some(419));
        assert_eq!(solve(NONCE, 16), Some(204_333));
    }
}
