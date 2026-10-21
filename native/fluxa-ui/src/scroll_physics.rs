use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::Duration;

use crate::UiPlatform;

const MAX_FLING_DP: f32 = 8000.0;
const MIN_FLING_DP: f32 = 50.0;
const VELOCITY_HORIZON: f32 = 0.1;
const POINTER_STOPPED: f32 = 0.04;

const SCROLL_FRICTION: f32 = 0.015;
const INFLEXION: f32 = 0.35;
const START_TENSION: f32 = 0.5;
const END_TENSION: f32 = 1.0;
const SPLINE_SAMPLES: usize = 100;

const IOS_DECELERATION: f32 = 0.998;
const IOS_STOP_SPEED: f32 = 2.0;
const RUBBER_COEFFICIENT: f32 = 0.55;
const SPRING_OMEGA: f32 = 14.0;
const EDGE_ENERGY: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollFeel {
    Android,
    Ios,
}

impl ScrollFeel {
    pub fn for_platform(platform: UiPlatform) -> Self {
        match platform {
            UiPlatform::Ios | UiPlatform::Macos => Self::Ios,
            _ => Self::Android,
        }
    }

    pub fn touch_slop(self) -> f32 {
        match self {
            Self::Android => 8.0,
            Self::Ios => 10.0,
        }
    }

    pub fn overscrolls(self) -> bool {
        self == Self::Ios
    }
}

#[derive(Debug, Default)]
pub struct VelocityTracker {
    samples: VecDeque<(f32, [f32; 2])>,
}

impl VelocityTracker {
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn add(&mut self, seconds: f32, position: [f32; 2]) {
        self.samples.push_back((seconds, position));
        while self.samples.len() > 32 {
            self.samples.pop_front();
        }
    }

    pub fn velocity(&self, axis: usize) -> f32 {
        let Some(&(newest, _)) = self.samples.back() else {
            return 0.0;
        };
        let mut points: Vec<(f32, f32)> = Vec::new();
        let mut next_at = newest;
        for &(at, position) in self.samples.iter().rev() {
            let age = newest - at;
            let gap = next_at - at;
            if age > VELOCITY_HORIZON || gap > POINTER_STOPPED {
                break;
            }
            points.push((-age, position[axis]));
            next_at = at;
        }
        points.reverse();
        impulse_velocity(&points)
    }

    pub fn fling_velocity(&self, axis: usize) -> f32 {
        self.velocity(axis).clamp(-MAX_FLING_DP, MAX_FLING_DP)
    }
}

fn kinetic_velocity(energy: f32) -> f32 {
    energy.signum() * (2.0 * energy.abs()).sqrt()
}

fn impulse_velocity(points: &[(f32, f32)]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut work = 0.0;
    for i in 1..points.len() {
        let dt = points[i].0 - points[i - 1].0;
        if dt <= 0.0 {
            continue;
        }
        let previous = kinetic_velocity(work);
        let current = (points[i].1 - points[i - 1].1) / dt;
        work += (current - previous) * current.abs();
        if i == 1 {
            work *= 0.5;
        }
    }
    kinetic_velocity(work)
}

fn spline() -> &'static [f32; SPLINE_SAMPLES + 1] {
    static TABLE: OnceLock<[f32; SPLINE_SAMPLES + 1]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let p1 = START_TENSION * INFLEXION;
        let p2 = 1.0 - END_TENSION * (1.0 - INFLEXION);
        let mut table = [0.0; SPLINE_SAMPLES + 1];
        for (i, slot) in table.iter_mut().enumerate().take(SPLINE_SAMPLES) {
            let alpha = i as f32 / SPLINE_SAMPLES as f32;
            let (mut lo, mut hi) = (0.0f32, 1.0f32);
            let mut x;
            let mut coef;
            loop {
                x = (lo + hi) / 2.0;
                coef = 3.0 * x * (1.0 - x);
                let tx = coef * ((1.0 - x) * p1 + x * p2) + x * x * x;
                if (tx - alpha).abs() < 1e-5 {
                    break;
                }
                if tx > alpha {
                    hi = x;
                } else {
                    lo = x;
                }
            }
            *slot = coef * ((1.0 - x) * START_TENSION + x * END_TENSION) + x * x * x;
        }
        table[SPLINE_SAMPLES] = 1.0;
        table
    })
}

fn spline_position(fraction: f32) -> f32 {
    let table = spline();
    let scaled = fraction.clamp(0.0, 1.0) * SPLINE_SAMPLES as f32;
    let index = (scaled as usize).min(SPLINE_SAMPLES - 1);
    let local = scaled - index as f32;
    table[index] + local * (table[index + 1] - table[index])
}

fn android_deceleration_rate() -> f32 {
    0.78f32.ln() / 0.9f32.ln()
}

fn android_fling(velocity: f32, density: f32) -> (f32, f32) {
    let ppi = 160.0 * density;
    let coefficient = 9.80665 * 39.37 * ppi * 0.84;
    let rate = android_deceleration_rate();
    let speed = velocity.abs() * density;
    let l = (INFLEXION * speed / (SCROLL_FRICTION * coefficient)).ln();
    let distance = SCROLL_FRICTION * coefficient * (rate / (rate - 1.0) * l).exp() / density;
    let duration = (l / (rate - 1.0)).exp();
    (distance, duration)
}

pub fn rubber_band(overshoot: f32, extent: f32) -> f32 {
    let extent = extent.max(1.0);
    extent * (1.0 - 1.0 / (overshoot * RUBBER_COEFFICIENT / extent + 1.0))
}

fn rubber_band_inverse(displayed: f32, extent: f32) -> f32 {
    let extent = extent.max(1.0);
    let displayed = displayed.min(extent * 0.98);
    extent / RUBBER_COEFFICIENT * displayed / (extent - displayed)
}

pub fn drag(feel: ScrollFeel, offset: f32, delta: f32, max: f32, extent: f32) -> f32 {
    if !feel.overscrolls() {
        return (offset + delta).clamp(0.0, max.max(0.0));
    }
    let max = max.max(0.0);
    let raw = if offset < 0.0 {
        -rubber_band_inverse(-offset, extent)
    } else if offset > max {
        max + rubber_band_inverse(offset - max, extent)
    } else {
        offset
    };
    let raw = raw + delta;
    if raw < 0.0 {
        -rubber_band(-raw, extent)
    } else if raw > max {
        max + rubber_band(raw - max, extent)
    } else {
        raw
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum Kind {
    #[default]
    Idle,
    AndroidFling {
        start: f32,
        distance: f32,
        duration: f32,
    },
    IosFling {
        start: f32,
        velocity: f32,
    },
    Spring {
        edge: f32,
        offset: f32,
        velocity: f32,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollMotion {
    kind: Kind,
    elapsed: f32,
    max: f32,
    feel: Option<ScrollFeel>,
}

impl ScrollMotion {
    pub fn is_active(&self) -> bool {
        !matches!(self.kind, Kind::Idle)
    }

    pub fn stop(&mut self) {
        self.kind = Kind::Idle;
        self.elapsed = 0.0;
    }

    pub fn release(
        &mut self,
        feel: ScrollFeel,
        offset: f32,
        velocity: f32,
        max: f32,
        density: f32,
    ) {
        self.elapsed = 0.0;
        self.max = max.max(0.0);
        self.feel = Some(feel);
        if offset < 0.0 || offset > self.max {
            let edge = if offset < 0.0 { 0.0 } else { self.max };
            self.kind = Kind::Spring {
                edge,
                offset: offset - edge,
                velocity: velocity * EDGE_ENERGY,
            };
            return;
        }
        if velocity.abs() < MIN_FLING_DP {
            self.kind = Kind::Idle;
            return;
        }
        self.kind = match feel {
            ScrollFeel::Android => {
                let (distance, duration) = android_fling(velocity, density);
                Kind::AndroidFling {
                    start: offset,
                    distance: distance * velocity.signum(),
                    duration,
                }
            }
            ScrollFeel::Ios => Kind::IosFling {
                start: offset,
                velocity,
            },
        };
    }

    pub fn step(&mut self, dt: Duration) -> Option<f32> {
        self.elapsed += dt.as_secs_f32();
        let t = self.elapsed;
        match self.kind {
            Kind::Idle => None,
            Kind::AndroidFling {
                start,
                distance,
                duration,
            } => {
                let value = start + distance * spline_position(t / duration);
                if value <= 0.0 || value >= self.max || t >= duration {
                    self.kind = Kind::Idle;
                }
                Some(value.clamp(0.0, self.max))
            }
            Kind::IosFling { start, velocity } => {
                let ln = IOS_DECELERATION.ln();
                let decay = (1000.0 * t * ln).exp();
                let value = start + velocity * (decay - 1.0) / (1000.0 * ln);
                let speed = velocity * decay;
                if value < 0.0 || value > self.max {
                    let edge = if value < 0.0 { 0.0 } else { self.max };
                    self.kind = Kind::Spring {
                        edge,
                        offset: value - edge,
                        velocity: speed * EDGE_ENERGY,
                    };
                    self.elapsed = 0.0;
                    return Some(value);
                }
                if speed.abs() < IOS_STOP_SPEED {
                    self.kind = Kind::Idle;
                }
                Some(value)
            }
            Kind::Spring {
                edge,
                offset,
                velocity,
            } => {
                let w = SPRING_OMEGA;
                let decay = (-w * t).exp();
                let x = (offset + (velocity + w * offset) * t) * decay;
                let v = (velocity - w * (velocity + w * offset) * t) * decay;
                if x.abs() < 0.3 && v.abs() < 4.0 {
                    self.kind = Kind::Idle;
                    return Some(edge);
                }
                Some(edge + x)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker(samples: &[(u64, f32)]) -> VelocityTracker {
        let mut tracker = VelocityTracker::default();
        for &(ms, y) in samples {
            tracker.add(ms as f32 / 1000.0, [0.0, y]);
        }
        tracker
    }

    #[test]
    fn steady_drag_reports_its_speed() {
        let tracker = tracker(&[(0, 0.0), (16, 8.0), (32, 16.0), (48, 24.0), (64, 32.0)]);
        let v = tracker.velocity(1);
        assert!((v - 500.0).abs() < 25.0, "{v}");
    }

    #[test]
    fn holding_still_before_release_cancels_the_fling() {
        let tracker = tracker(&[(0, 0.0), (16, 8.0), (32, 16.0), (120, 16.0)]);
        assert_eq!(tracker.velocity(1), 0.0);
    }

    #[test]
    fn android_fling_travels_further_when_faster() {
        let (slow, _) = android_fling(1000.0, 2.75);
        let (fast, _) = android_fling(4000.0, 2.75);
        assert!(fast > slow * 3.0);
    }

    #[test]
    fn android_fling_stops_at_the_end() {
        let mut motion = ScrollMotion::default();
        motion.release(ScrollFeel::Android, 100.0, 6000.0, 300.0, 2.75);
        let mut last = 0.0;
        for _ in 0..240 {
            if let Some(value) = motion.step(Duration::from_millis(16)) {
                last = value;
            }
        }
        assert_eq!(last, 300.0);
        assert!(!motion.is_active());
    }

    #[test]
    fn ios_spring_returns_to_the_edge() {
        let mut motion = ScrollMotion::default();
        motion.release(ScrollFeel::Ios, -80.0, 0.0, 300.0, 3.0);
        let mut last = -80.0;
        for _ in 0..120 {
            if let Some(value) = motion.step(Duration::from_millis(16)) {
                last = value;
            }
        }
        assert_eq!(last, 0.0);
    }

    #[test]
    fn rubber_band_never_exceeds_the_extent() {
        assert!(rubber_band(10_000.0, 800.0) < 800.0);
        let pulled = drag(ScrollFeel::Ios, 0.0, -200.0, 500.0, 800.0);
        assert!(pulled < 0.0 && pulled > -200.0);
    }
}
