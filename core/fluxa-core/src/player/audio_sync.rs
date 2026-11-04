use serde::Deserialize;
use serde_json::json;

const KNOWN_RATES: [f64; 9] = [
    1.0,
    25.0 / 23.976,
    23.976 / 25.0,
    24.0 / 23.976,
    23.976 / 24.0,
    25.0 / 24.0,
    24.0 / 25.0,
    30.0 / 29.97,
    29.97 / 30.0,
];

const MIN_SCORE: f64 = 0.25;
const KNOWN_RATE_PREFERENCE: f64 = 0.85;
const INLIER_SECONDS: f64 = 0.3;
const MIN_FREE_FIT_HITS: usize = 3;

#[derive(Clone, Debug)]
pub(crate) struct Envelope {
    pub start: f64,
    pub frame: f64,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug)]
pub(crate) struct Window {
    pub reference: Envelope,
    pub external: Envelope,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AudioSync {
    pub rate: f64,
    pub offset: f64,
    pub confidence: f64,
}

struct Hit {
    time: f64,
    offset: f64,
    score: f64,
}

impl Envelope {
    fn at(&self, time: f64) -> Option<f64> {
        let position = (time - self.start) / self.frame;
        if position < 0.0 {
            return None;
        }
        let index = position.floor() as usize;
        let next = self.values.get(index + 1)?;
        let fraction = position - index as f64;
        Some(self.values[index] * (1.0 - fraction) + next * fraction)
    }

    fn end(&self) -> f64 {
        self.start + self.frame * self.values.len().saturating_sub(1) as f64
    }
}

pub(crate) fn estimate(windows: &[Window]) -> Option<AudioSync> {
    let mut rates: Vec<(f64, bool)> = KNOWN_RATES.iter().map(|rate| (*rate, true)).collect();
    let flat = hits(windows, 1.0);
    if flat.len() >= MIN_FREE_FIT_HITS {
        if let Some(slope) = theil_sen(&flat) {
            let rate = 1.0 + slope;
            if (0.9..=1.1).contains(&rate) && KNOWN_RATES.iter().all(|r| (r - rate).abs() > 0.0003)
            {
                rates.push((rate, false));
            }
        }
    }
    let scored: Vec<(AudioSync, f64, bool)> = rates
        .into_iter()
        .filter_map(|(rate, known)| {
            evaluate(windows, rate).map(|(sync, total)| (sync, total, known))
        })
        .collect();
    let best = scored
        .iter()
        .map(|(_, total, _)| *total)
        .fold(0.0, f64::max);
    scored
        .into_iter()
        .filter(|(_, total, known)| *known && *total >= best * KNOWN_RATE_PREFERENCE)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(sync, _, _)| sync)
        .or_else(|| {
            let hits = hits(windows, 1.0);
            let slope = theil_sen(&hits)?;
            evaluate(windows, 1.0 + slope).map(|(sync, _)| sync)
        })
}

fn evaluate(windows: &[Window], rate: f64) -> Option<(AudioSync, f64)> {
    let found = hits(windows, rate);
    if found.is_empty() {
        return None;
    }
    let mut offsets: Vec<f64> = found.iter().map(|hit| hit.offset).collect();
    offsets.sort_by(f64::total_cmp);
    let median = offsets[offsets.len() / 2];
    let inliers: Vec<&Hit> = found
        .iter()
        .filter(|hit| (hit.offset - median).abs() <= INLIER_SECONDS)
        .collect();
    let needed = if windows.len() >= 3 { 2 } else { 1 };
    if inliers.len() < needed || inliers.len() * 2 < windows.len() {
        return None;
    }
    let total: f64 = inliers.iter().map(|hit| hit.score).sum();
    let weighted = inliers
        .iter()
        .map(|hit| hit.offset * hit.score)
        .sum::<f64>()
        / total;
    let confidence = total / windows.len() as f64;
    let sync = AudioSync {
        rate,
        offset: (weighted * 100.0).round() / 100.0,
        confidence,
    };
    Some((sync, total))
}

fn hits(windows: &[Window], rate: f64) -> Vec<Hit> {
    windows
        .iter()
        .filter_map(|window| {
            let (offset, score) = best_offset(window, rate)?;
            let reference = &window.reference;
            Some(Hit {
                time: (reference.start + reference.end()) / 2.0,
                offset,
                score,
            })
        })
        .collect()
}

fn best_offset(window: &Window, rate: f64) -> Option<(f64, f64)> {
    let reference = &window.reference;
    let external = &window.external;
    let count = reference.values.len();
    if count < 8 {
        return None;
    }
    let mean = reference.values.iter().sum::<f64>() / count as f64;
    let centered: Vec<f64> = reference.values.iter().map(|v| v - mean).collect();
    let norm = centered.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm < 1e-9 {
        return None;
    }
    let last = reference.start + reference.frame * (count - 1) as f64;
    let low = external.start - rate * reference.start;
    let high = external.end() - rate * last;
    if high <= low {
        return None;
    }
    let step = reference.frame;
    let steps = ((high - low) / step).floor() as usize;
    let mut scores = Vec::with_capacity(steps + 1);
    let mut sampled = vec![0.0; count];
    for n in 0..=steps {
        let offset = low + n as f64 * step;
        let mut valid = true;
        for (i, slot) in sampled.iter_mut().enumerate() {
            let time = rate * (reference.start + i as f64 * reference.frame) + offset;
            match external.at(time) {
                Some(value) => *slot = value,
                None => {
                    valid = false;
                    break;
                }
            }
        }
        scores.push(if valid {
            correlation(&centered, norm, &sampled)
        } else {
            -1.0
        });
    }
    let (best, &score) = scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))?;
    if score < MIN_SCORE {
        return None;
    }
    let mut position = best as f64;
    if best > 0 && best + 1 < scores.len() {
        let (left, right) = (scores[best - 1], scores[best + 1]);
        let curve = left - 2.0 * score + right;
        if curve < -1e-9 {
            position += 0.5 * (left - right) / curve;
        }
    }
    Some((low + position * step, score))
}

fn correlation(centered: &[f64], norm: f64, other: &[f64]) -> f64 {
    let mean = other.iter().sum::<f64>() / other.len() as f64;
    let mut dot = 0.0;
    let mut energy = 0.0;
    for (a, b) in centered.iter().zip(other) {
        let b = b - mean;
        dot += a * b;
        energy += b * b;
    }
    if energy < 1e-12 {
        return -1.0;
    }
    dot / (norm * energy.sqrt())
}

fn theil_sen(hits: &[Hit]) -> Option<f64> {
    let mut slopes = Vec::new();
    for (i, a) in hits.iter().enumerate() {
        for b in &hits[i + 1..] {
            let gap = b.time - a.time;
            if gap.abs() > 30.0 {
                slopes.push((b.offset - a.offset) / gap);
            }
        }
    }
    if slopes.is_empty() {
        return None;
    }
    slopes.sort_by(f64::total_cmp);
    Some(slopes[slopes.len() / 2])
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnergyRequest {
    start: f64,
    frame_seconds: f64,
    energies: Vec<f64>,
}

#[derive(Deserialize)]
struct WindowRequest {
    reference: EnergyRequest,
    external: EnergyRequest,
}

#[derive(Deserialize)]
struct EstimateRequest {
    windows: Vec<WindowRequest>,
}

impl From<EnergyRequest> for Envelope {
    fn from(energy: EnergyRequest) -> Self {
        Envelope {
            start: energy.start,
            frame: energy.frame_seconds,
            values: energy
                .energies
                .into_iter()
                .map(|value| (value * 50.0).ln_1p())
                .collect(),
        }
    }
}

pub(crate) fn audio_sync_estimate_json(request_json: &str) -> Option<String> {
    let request: EstimateRequest = serde_json::from_str(request_json).ok()?;
    let windows: Vec<Window> = request
        .windows
        .into_iter()
        .map(|window| Window {
            reference: window.reference.into(),
            external: window.external.into(),
        })
        .collect();
    let sync = estimate(&windows)?;
    Some(
        json!({
            "rate": sync.rate,
            "offset": sync.offset,
            "confidence": sync.confidence,
        })
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f64 = 0.05;
    const SPAN: f64 = 20.0;

    fn events(seed: u64, duration: f64) -> Vec<(f64, f64)> {
        let mut state = seed;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as f64 / (1u64 << 31) as f64
        };
        let mut time = 0.0;
        let mut out = Vec::new();
        while time < duration {
            time += 0.2 + next() * 1.2;
            out.push((time, 0.3 + next()));
        }
        out
    }

    fn level(events: &[(f64, f64)], shift: f64, rate: f64, time: f64) -> f64 {
        events
            .iter()
            .map(|(at, weight)| {
                let distance = (time - (rate * at + shift)) / 0.12;
                weight * (-distance * distance).exp()
            })
            .sum()
    }

    fn windows(events: &[(f64, f64)], other: &[(f64, f64)], rate: f64, offset: f64) -> Vec<Window> {
        [10.0, 200.0, 420.0, 700.0, 960.0]
            .into_iter()
            .map(|start| {
                let reference = Envelope {
                    start,
                    frame: FRAME,
                    values: (0..(SPAN / FRAME) as usize)
                        .map(|i| level(events, 0.0, 1.0, start + i as f64 * FRAME))
                        .collect(),
                };
                let from = rate * start + offset - 6.0;
                let external = Envelope {
                    start: from,
                    frame: FRAME,
                    values: (0..((SPAN * rate + 12.0) / FRAME) as usize)
                        .map(|i| level(other, offset, rate, from + i as f64 * FRAME))
                        .collect(),
                };
                Window {
                    reference,
                    external,
                }
            })
            .collect()
    }

    #[test]
    fn pal_speedup_with_offset_is_recovered() {
        let reference = events(7, 1100.0);
        let rate = 25.0 / 23.976;
        let sync = estimate(&windows(&reference, &reference, rate, 0.6)).unwrap();
        assert!((sync.rate - rate).abs() < 1e-9);
        assert!((sync.offset - 0.6).abs() < 0.06, "{}", sync.offset);
    }

    #[test]
    fn same_speed_negative_offset() {
        let reference = events(11, 1100.0);
        let sync = estimate(&windows(&reference, &reference, 1.0, -2.3)).unwrap();
        assert_eq!(sync.rate, 1.0);
        assert!((sync.offset + 2.3).abs() < 0.06, "{}", sync.offset);
    }

    #[test]
    fn unlisted_rate_is_fitted() {
        let reference = events(3, 1100.0);
        let sync = estimate(&windows(&reference, &reference, 1.02, 1.4)).unwrap();
        assert!((sync.rate - 1.02).abs() < 0.0005, "{}", sync.rate);
        assert!((sync.offset - 1.4).abs() < 0.15, "{}", sync.offset);
    }

    #[test]
    fn unrelated_audio_is_rejected() {
        let reference = events(5, 1100.0);
        let other = events(99, 1100.0);
        assert!(estimate(&windows(&reference, &other, 1.0, 0.0)).is_none());
    }
}
