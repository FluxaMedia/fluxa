#[derive(Clone, Debug, Default)]
pub struct PartyClock {
    anchor: Option<(i64, i64)>,
    best_round_trip_ms: Option<i64>,
}

impl PartyClock {
    pub fn is_synced(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn now_ms(&self, local_ms: i64, wall_ms: i64) -> i64 {
        match self.anchor {
            Some((server_ms, anchored_at)) => server_ms + (local_ms - anchored_at),
            None => wall_ms,
        }
    }

    pub fn add_sample(&mut self, server_ms: i64, sent_at: i64, received_at: i64) {
        let round_trip = (received_at - sent_at).max(0);
        if self.best_round_trip_ms.is_some_and(|best| round_trip > best) {
            return;
        }
        self.best_round_trip_ms = Some(round_trip);
        self.anchor = Some((server_ms + round_trip / 2, received_at));
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_wall_clock_until_sampled() {
        let clock = PartyClock::default();
        assert_eq!(clock.now_ms(500, 9_000), 9_000);
    }

    #[test]
    fn keeps_the_lowest_round_trip_sample() {
        let mut clock = PartyClock::default();
        clock.add_sample(10_000, 0, 100);
        clock.add_sample(99_999, 200, 900);
        assert_eq!(clock.now_ms(100, 0), 10_050);
        clock.add_sample(20_000, 1_000, 1_020);
        assert_eq!(clock.now_ms(1_020, 0), 20_010);
    }
}
