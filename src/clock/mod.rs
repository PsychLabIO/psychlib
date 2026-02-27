pub use instant::{Duration, FrameTimestamp, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::trace;

mod instant;
mod platform;

/// `Clock` is cheap to clone — it's an `Arc` internally. A single `Clock`
/// instance should be created at experiment start and shared across the
/// trial runner, scheduler, and data recorder so all timestamps share
/// the same epoch.
#[derive(Clone, Debug)]
pub struct Clock {
    inner: Arc<ClockInner>,
}

#[derive(Debug)]
struct ClockInner {
    epoch: platform::RawInstant,
    epoch_wall: DateTime<Utc>,
    frame_log: Mutex<Vec<FrameTimestamp>>,
}

impl Clock {
    pub fn new() -> Self {
        let epoch = platform::RawInstant::now();
        let epoch_wall = Utc::now();
        trace!("Clock epoch set: {:?}", epoch_wall);

        Self {
            inner: Arc::new(ClockInner {
                epoch,
                epoch_wall,
                frame_log: Mutex::new(Vec::new()),
            }),
        }
    }

    #[inline]
    pub fn now(&self) -> Instant {
        let raw = platform::RawInstant::now();
        let nanos = raw.nanos_since(&self.inner.epoch);
        Instant::from_nanos(nanos)
    }

    #[inline]
    pub fn elapsed(&self, start: Instant) -> Duration {
        self.now() - start
    }

    pub fn to_wall_time(&self, instant: Instant) -> DateTime<Utc> {
        let offset = chrono::Duration::nanoseconds(instant.as_nanos() as i64);
        self.inner.epoch_wall + offset
    }

    pub fn epoch_wall(&self) -> DateTime<Utc> {
        self.inner.epoch_wall
    }

    /// Returns the actual `Instant` when the thread woke up, which will
    /// be slightly after `target` due to OS scheduler jitter.
    pub fn sleep_until(&self, target: Instant) -> Instant {
        let now = self.now();
        if target <= now {
            return now;
        }
        let remaining = target - now;
        platform::sleep(remaining);
        self.now()
    }

    pub fn sleep(&self, duration: Duration) -> Instant {
        let target = self.now() + duration;
        self.sleep_until(target)
    }

    pub fn record_frame(&self, label: impl Into<String>) -> FrameTimestamp {
        let ts = FrameTimestamp {
            instant: self.now(),
            wall: self.to_wall_time(self.now()),
            label: label.into(),
        };
        self.inner
            .frame_log
            .lock()
            .expect("frame log mutex poisoned")
            .push(ts.clone());
        ts
    }

    /// Return a snapshot of all recorded frame timestamps so far.
    pub fn frame_log(&self) -> Vec<FrameTimestamp> {
        self.inner
            .frame_log
            .lock()
            .expect("frame log mutex poisoned")
            .clone()
    }

    /// Clear the frame log (e.g. between trials).
    pub fn clear_frame_log(&self) {
        self.inner
            .frame_log
            .lock()
            .expect("frame log mutex poisoned")
            .clear();
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// A serializable snapshot of clock metadata, written into data files so
/// timing can always be reconstructed offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockInfo {
    /// ISO 8601 wall-clock epoch for this session.
    pub epoch_wall: DateTime<Utc>,

    /// Platform string, e.g. "native-linux", "wasm-chrome".
    pub platform: String,

    /// Whether sub-millisecond spin-sleep was available.
    pub high_precision_sleep: bool,
}

impl Clock {
    pub fn info(&self) -> ClockInfo {
        ClockInfo {
            epoch_wall: self.inner.epoch_wall,
            platform: platform::PLATFORM_STR.to_string(),
            high_precision_sleep: platform::HIGH_PRECISION_SLEEP,
        }
    }
}
