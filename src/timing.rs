//! Utilities for measuring time.

use std::time::Duration;

/// Run the `rdtsc` instruction and return the value
#[inline(always)]
pub fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;

    unsafe {
        asm!("rdtsc" : "={eax}"(lo), "={edx}"(hi));
    }

    u64::from(lo) | (u64::from(hi) << 32)
}

/// Like std::time::Instant but for rdtsc.
pub struct Tsc {
    tsc: u64,
    freq: Option<usize>,
}

impl Tsc {
    /// Capture the TSC now.
    pub fn now() -> Self {
        Tsc {
            tsc: rdtsc(),
            freq: None,
        }
    }

    /// Set the frequency of this `Tsc`. You need to do this before using `duration_since`;
    /// otherwise, we have no way to convert to seconds. `freq` should be in MHz.
    pub fn set_freq(&mut self, freq: usize) {
        self.freq = Some(freq);
    }

    /// Returns a `Duration` representing the time since `earlier`.
    ///
    /// # Panics
    ///
    /// If `earlier` is not `earlier`.
    pub fn duration_since(&self, earlier: Self) -> Duration {
        assert!(earlier.tsc < self.tsc);

        let diff = self.tsc - earlier.tsc;
        let nanos = diff * 1000 / self.freq.unwrap() as u64;

        Duration::from_nanos(nanos)
    }
}
