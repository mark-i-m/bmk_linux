//! Utilities for measuring time.

use std::{collections::HashMap, time::Duration, arch::asm};

/// A trait for time sources. This allows methods to be generic over different ways of measuring
/// time.
pub trait Clock {
    /// Get a timestamp.
    fn now() -> Self;

    /// Set a scaling factor, which can be used to convert a difference of timestamps to seconds.
    fn set_scaling_factor(&mut self, scaling: usize);

    /// Get the duration from `earlier` to `self` as a `std::time::Duration`.
    ///
    /// # Panics
    ///
    /// If `earlier` is not `earlier`.
    fn duration_since(&self, earlier: Self) -> Duration;
}

/// Run the `rdtsc` instruction and return the value
#[inline(always)]
pub fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;

    unsafe {
        asm!("rdtsc", out("eax") lo, out("edx") hi);
    }

    u64::from(lo) | (u64::from(hi) << 32)
}

/// Like std::time::Instant but for rdtsc.
pub struct Tsc {
    tsc: u64,
    freq: Option<usize>,
}

impl Clock for Tsc {
    #[inline(always)]
    fn now() -> Self {
        Tsc {
            tsc: rdtsc(),
            freq: None,
        }
    }

    fn set_scaling_factor(&mut self, freq: usize) {
        self.freq = Some(freq);
    }

    fn duration_since(&self, earlier: Self) -> Duration {
        assert!(earlier.tsc < self.tsc);

        let diff = self.tsc - earlier.tsc;
        let nanos = diff * 1000 / self.freq.unwrap() as u64;

        Duration::from_nanos(nanos)
    }
}

impl Clock for std::time::Instant {
    #[inline(always)]
    fn now() -> Self {
        std::time::Instant::now()
    }

    fn set_scaling_factor(&mut self, _freq: usize) {
        // nop because we are already in seconds...
    }

    fn duration_since(&self, earlier: Self) -> Duration {
        self.duration_since(earlier)
    }
}

#[cfg(test)]
mod test {
    use super::Clock;
    use std::time::Instant;

    #[test]
    fn test_instant_clock() {
        let earlier = Instant::now();
        let _ = <Instant as Clock>::duration_since(&Instant::now(), earlier);
    }
}

#[derive(Default, Debug)]
pub struct MemoizedTimingData {
    cached_avg: Option<f64>,
    cached_sd: Option<f64>,
    cached_max: Option<f64>,

    cached_sorted: Option<Vec<u64>>,
    cached_percentiles: HashMap<usize, f64>,
}

impl MemoizedTimingData {
    pub fn new() -> Self {
        Self {
            cached_avg: None,
            cached_sd: None,
            cached_max: None,

            cached_sorted: None,
            cached_percentiles: HashMap::new(),
        }
    }

    pub fn avg(&mut self, measurements: &[u64]) -> f64 {
        if let Some(avg) = self.cached_avg {
            return avg;
        }

        let n = measurements.len();
        let sum: u64 = measurements.iter().sum();

        let avg = (sum as f64) / (n as f64);

        self.cached_avg = Some(avg);

        avg
    }

    pub fn sd(&mut self, measurements: &[u64]) -> f64 {
        if let Some(sd) = self.cached_sd {
            return sd;
        }

        let n = measurements.len() as f64;
        let avg = self.avg(measurements);
        let deviations_sq: f64 = measurements.iter().map(|&x| (x as f64 - avg).powi(2)).sum();
        let sd = (deviations_sq / n).sqrt();

        self.cached_sd = Some(sd);

        sd
    }

    fn sorted_data(&mut self, measurements: &[u64]) -> &Vec<u64> {
        if let Some(ref sorted) = self.cached_sorted {
            return sorted;
        }

        let mut clone = measurements.to_vec();
        clone.sort_unstable();
        self.cached_sorted = Some(clone);

        self.cached_sorted.as_ref().unwrap()
    }

    pub fn percentile(&mut self, measurements: &[u64], percentile: usize) -> f64 {
        assert!(percentile < 100);

        if let Some(&percentile) = self.cached_percentiles.get(&percentile) {
            return percentile;
        }

        let val = {
            let sorted = self.sorted_data(measurements);
            let idx = sorted.len() * percentile / 100;
            //println!("[debug] {} {}", percentile, idx);
            assert!(idx < sorted.len());

            sorted[idx] as f64
        };

        self.cached_percentiles.insert(percentile, val);

        val
    }

    pub fn permicrotile(&mut self, measurements: &[u64], permicrotile: usize) -> f64 {
        assert!(permicrotile < 1_000_000 && permicrotile > 990_000);

        if let Some(&permicrotile) = self.cached_percentiles.get(&permicrotile) {
            return permicrotile;
        }

        let val = {
            let sorted = self.sorted_data(measurements);
            let idx = sorted.len() * permicrotile / 1_000_000;
            //println!("[debug] {} {}", permicrotile, idx);
            assert!(idx < sorted.len());

            sorted[idx] as f64
        };

        self.cached_percentiles.insert(permicrotile, val);

        val
    }

    pub fn max(&mut self, measurements: &[u64]) -> f64 {
        if let Some(max) = self.cached_max {
            return max;
        }

        let val = {
            let sorted = self.sorted_data(measurements);
            *sorted.last().unwrap() as f64
        };

        self.cached_max = Some(val);

        val
    }
}
