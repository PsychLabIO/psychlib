use crate::clock::instant::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub const PLATFORM_STR: &str = {
    #[cfg(target_os = "linux")]
    {
        "native-linux"
    }
    #[cfg(target_os = "macos")]
    {
        "native-macos"
    }
    #[cfg(target_os = "windows")]
    {
        "native-windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "native-unknown"
    }
};

#[cfg(target_arch = "wasm32")]
pub const PLATFORM_STR: &str = "wasm";

#[cfg(not(target_arch = "wasm32"))]
pub const HIGH_PRECISION_SLEEP: bool = true;

#[cfg(target_arch = "wasm32")]
pub const HIGH_PRECISION_SLEEP: bool = false;

/// - **Native**: `std::time::Instant` (backed by CLOCK_MONOTONIC_RAW on Linux,
///   mach_absolute_time on macOS, QueryPerformanceCounter on Windows).
/// - **WASM**: `performance.now()` via `web-sys` (f64 milliseconds, ~5µs resolution
///   in most browsers; reduced to 100µs/1ms in cross-origin-isolated contexts).
#[derive(Clone, Copy, Debug)]
pub struct RawInstant {
    #[cfg(not(target_arch = "wasm32"))]
    inner: std::time::Instant,

    #[cfg(target_arch = "wasm32")]
    millis: f64,
}

impl RawInstant {
    pub fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                inner: std::time::Instant::now(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().expect("no window object");
            let perf = window.performance().expect("no performance object");
            Self { millis: perf.now() }
        }
    }

    pub fn nanos_since(&self, earlier: &Self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner
                .checked_duration_since(earlier.inner)
                .unwrap_or(std::time::Duration::ZERO)
                .as_nanos() as u64
        }

        #[cfg(target_arch = "wasm32")]
        {
            let diff_ms = self.millis - earlier.millis;
            if diff_ms <= 0.0 {
                0
            } else {
                (diff_ms * 1_000_000.0) as u64
            }
        }
    }
}

/// Sleep for `duration` with the best available precision on this platform.
/// ## Native strategy
/// Uses a two-phase approach:
/// 1. `std::thread::sleep` for the bulk of the duration (OS scheduler, ~50–500µs jitter).
/// 2. `spin_sleep::sleep` (or a raw spin loop) for the final 2 ms, burning CPU
///    to eliminate OS wakeup latency. This achieves ~10–50µs accuracy.
/// The spin threshold can be tuned per-platform; 2 ms is conservative and
/// works well on both Linux (`CLOCK_MONOTONIC`) and macOS (`mach_timebase`).
/// ## WASM strategy
/// Sleeping is not meaningful in a WASM context, the browser drives the
/// frame loop via `requestAnimationFrame`. This function is a no-op on WASM;
/// the scheduler instead calculates target frame counts.
pub fn sleep(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    native_sleep(duration);

    #[cfg(target_arch = "wasm32")]
    {
        let _ = duration;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_sleep(duration: Duration) {
    use std::time::Duration;

    if duration.is_negative() || duration.is_zero() {
        return;
    }

    let nanos = duration.as_nanos() as u64;

    const SPIN_THRESHOLD_NS: u64 = 2_000_000;

    if nanos <= SPIN_THRESHOLD_NS {
        spin_loop(nanos);
        return;
    }

    let sleep_nanos = nanos - SPIN_THRESHOLD_NS;

    #[cfg(feature = "native")]
    spin_sleep::sleep(Duration::from_nanos(sleep_nanos));

    #[cfg(not(feature = "native"))]
    std::thread::sleep(Duration::from_nanos(sleep_nanos));

    spin_loop(SPIN_THRESHOLD_NS);
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
fn spin_loop(nanos: u64) {
    let start = std::time::Instant::now();
    let target = std::time::Duration::from_nanos(nanos);
    while start.elapsed() < target {
        std::hint::spin_loop();
    }
}
