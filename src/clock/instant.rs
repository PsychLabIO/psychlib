use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A signed duration stored as integer nanoseconds.
/// Signed so that timing differences (e.g. `actual_onset - target_onset`) can
/// be negative when the event was early.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Duration {
    nanos: i64,
}

impl Duration {
    pub const ZERO: Self = Self { nanos: 0 };

    #[inline]
    pub fn from_nanos(nanos: i64) -> Self {
        Self { nanos }
    }
    #[inline]
    pub fn from_micros(micros: i64) -> Self {
        Self {
            nanos: micros * 1_000,
        }
    }
    #[inline]
    pub fn from_millis(millis: i64) -> Self {
        Self {
            nanos: millis * 1_000_000,
        }
    }
    #[inline]
    pub fn from_secs(secs: f64) -> Self {
        Self {
            nanos: (secs * 1e9) as i64,
        }
    }

    #[inline]
    pub fn as_nanos(self) -> i64 {
        self.nanos
    }
    #[inline]
    pub fn as_micros(self) -> i64 {
        self.nanos / 1_000
    }
    #[inline]
    pub fn as_millis(self) -> i64 {
        self.nanos / 1_000_000
    }
    #[inline]
    pub fn as_secs_f64(self) -> f64 {
        self.nanos as f64 / 1e9
    }
    #[inline]
    pub fn abs(self) -> Self {
        Self {
            nanos: self.nanos.abs(),
        }
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self.nanos == 0
    }
    #[inline]
    pub fn is_negative(self) -> bool {
        self.nanos < 0
    }

    /// Convert to `std::time::Duration`. Panics if duration is negative
    /// use only when you're sure the value is non-negative.
    pub fn to_std(self) -> std::time::Duration {
        assert!(
            self.nanos >= 0,
            "Duration::to_std called on negative duration ({}ns)",
            self.nanos
        );
        std::time::Duration::from_nanos(self.nanos as u64)
    }

    pub fn checked_to_std(self) -> Option<std::time::Duration> {
        if self.nanos >= 0 {
            Some(std::time::Duration::from_nanos(self.nanos as u64))
        } else {
            None
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.nanos.abs();
        let sign = if self.nanos < 0 { "-" } else { "" };
        if abs >= 1_000_000_000 {
            write!(f, "{}{:.3}s", sign, abs as f64 / 1e9)
        } else if abs >= 1_000_000 {
            write!(f, "{}{:.2}ms", sign, abs as f64 / 1e6)
        } else if abs >= 1_000 {
            write!(f, "{}{:.1}µs", sign, abs as f64 / 1e3)
        } else {
            write!(f, "{}{}ns", sign, abs)
        }
    }
}

impl Add for Duration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            nanos: self.nanos + rhs.nanos,
        }
    }
}
impl Sub for Duration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            nanos: self.nanos - rhs.nanos,
        }
    }
}
impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.nanos += rhs.nanos;
    }
}
impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        self.nanos -= rhs.nanos;
    }
}
impl std::ops::Neg for Duration {
    type Output = Self;
    fn neg(self) -> Self {
        Self { nanos: -self.nanos }
    }
}
impl std::ops::Mul<i64> for Duration {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Self {
            nanos: self.nanos * rhs,
        }
    }
}

/// Stored as `u64` so it's always non-negative and can safely represent
/// ~584 years of experiment time before overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    pub const ZERO: Self = Self { nanos: 0 };

    #[inline]
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
    #[inline]
    pub fn as_nanos(self) -> u64 {
        self.nanos
    }
    #[inline]
    pub fn as_secs_f64(self) -> f64 {
        self.nanos as f64 / 1e9
    }
    #[inline]
    pub fn as_millis(self) -> u64 {
        self.nanos / 1_000_000
    }
}

impl Sub for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        Duration::from_nanos(self.nanos as i64 - rhs.nanos as i64)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        let result = self.nanos as i64 + rhs.nanos;
        Self {
            nanos: result.max(0) as u64,
        }
    }
}
impl Sub<Duration> for Instant {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        let result = self.nanos as i64 - rhs.nanos;
        Self {
            nanos: result.max(0) as u64,
        }
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T+{:.6}s", self.nanos as f64 / 1e9)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTimestamp {
    pub instant: Instant,

    pub wall: DateTime<Utc>,

    pub label: String,
}

impl fmt::Display for FrameTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({})",
            self.label,
            self.instant,
            self.wall.format("%H:%M:%S%.6f")
        )
    }
}
