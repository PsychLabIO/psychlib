use crate::clock::{Clock, Duration, Instant};
use crate::io::keyboard::{KeyCode, KeyState, flush_key_buffer, poll_key_event};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::trace;

/// A participant's response: the key they pressed and when, relative to
/// the stimulus onset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub key: KeyCode,
    pub rt: Duration,
    pub timestamp: Instant,
    pub onset: Instant,
}

impl Response {
    /// RT in milliseconds as f64
    pub fn rt_ms(&self) -> f64 {
        self.rt.as_secs_f64() * 1000.0
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key={} rt={:.1}ms", self.key, self.rt_ms())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseOutcome {
    Response(Response),
    Timeout,
}

impl ResponseOutcome {
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response(_))
    }
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }
    pub fn response(self) -> Option<Response> {
        match self {
            Self::Response(r) => Some(r),
            Self::Timeout => None,
        }
    }
}

pub struct ResponseWindow<'a> {
    clock: &'a Clock,
    accept: Vec<KeyCode>,
    timeout: Option<Duration>,
    flush_on_arm: bool,
    onset: Option<Instant>,
}

impl<'a> ResponseWindow<'a> {
    pub fn new(clock: &'a Clock) -> Self {
        Self {
            clock,
            accept: Vec::new(),
            timeout: None,
            flush_on_arm: true,
            onset: None,
        }
    }

    /// Set the keys this window will accept.
    /// Calling with an empty slice means "accept any non-modifier key".
    pub fn accept_keys(mut self, keys: &[KeyCode]) -> Self {
        self.accept = keys.to_vec();
        self
    }

    /// Set the response timeout. After this duration from `arm()`, `wait()`
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn flush_on_arm(mut self, flush: bool) -> Self {
        self.flush_on_arm = flush;
        self
    }

    pub fn arm(&mut self) -> Instant {
        if self.flush_on_arm {
            flush_key_buffer();
        }
        let onset = self.clock.now();
        self.onset = Some(onset);
        trace!("ResponseWindow armed at {}", onset);
        onset
    }

    pub fn arm_at(&mut self, onset: Instant) -> Instant {
        if self.flush_on_arm {
            flush_key_buffer();
        }
        self.onset = Some(onset);
        trace!("ResponseWindow armed at (frame) {}", onset);
        onset
    }

    pub fn wait(self) -> ResponseOutcome {
        let onset = self
            .onset
            .expect("ResponseWindow::wait called before arm() - call arm() after stimulus onset");

        let deadline: Option<Instant> = self.timeout.map(|t| onset + t);

        loop {
            if let Some(deadline) = deadline {
                if self.clock.now() >= deadline {
                    trace!("ResponseWindow timed out");
                    return ResponseOutcome::Timeout;
                }
            }

            if let Some((code, state)) = poll_key_event() {
                if state != KeyState::Pressed {
                    continue;
                }

                if code.is_modifier() {
                    continue;
                }

                if !self.accept.is_empty() && !self.accept.contains(&code) {
                    trace!("ResponseWindow ignoring key {}", code);
                    continue;
                }

                let timestamp = self.clock.now();
                let rt = timestamp - onset;

                let rt = if rt.is_negative() { Duration::ZERO } else { rt };

                let response = Response {
                    key: code,
                    rt,
                    timestamp,
                    onset,
                };
                trace!("ResponseWindow: {}", response);
                return ResponseOutcome::Response(response);
            }
            std::hint::spin_loop();
        }
    }
}

impl<'a> ResponseWindow<'a> {
    pub fn wait_any_key(clock: &'a Clock, timeout: Duration) -> ResponseOutcome {
        let mut w = Self::new(clock).timeout(timeout);
        w.arm();
        w.wait()
    }

    pub fn wait_keys(clock: &'a Clock, keys: &[KeyCode], timeout: Duration) -> ResponseOutcome {
        let mut w = Self::new(clock).accept_keys(keys).timeout(timeout);
        w.arm();
        w.wait()
    }
}
