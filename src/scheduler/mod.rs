use crate::clock::{Clock, Duration, FrameTimestamp, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::{debug, warn};

/// What kind of thing is happening at this point in the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Blank,
    Fixation,
    StimulusOnset,
    StimulusOffset,
    ResponseWindowOpen,
    ResponseWindowClose,
    Mask,
    Iti,
    Feedback,
    Custom(String),
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "custom:{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialPhase {
    Practice,
    Experiment,
    Break,
    EndScreen,
}

/// A single scheduled event in the experiment timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub trial_index: usize,
    pub phase: TrialPhase,
    pub kind: EventKind,
    pub scheduled_at: Instant,
    pub actual_onset: Option<Instant>,
    pub actual_wall: Option<DateTime<Utc>>,
    pub frame: Option<FrameTimestamp>,
}

impl Event {
    pub fn timing_error(&self) -> Option<Duration> {
        self.actual_onset.map(|onset| onset - self.scheduled_at)
    }
}

/// Manages the timing of events within and across trials.
pub struct Scheduler {
    clock: Clock,
    next_id: u32,
    current_trial: usize,
    current_phase: TrialPhase,
    trial_start: Instant,
    cursor: Instant,
    pending: VecDeque<Event>,
    completed: Vec<Event>,
}

impl Scheduler {
    pub fn new(clock: Clock) -> Self {
        let now = clock.now();
        Self {
            clock,
            next_id: 0,
            current_trial: 0,
            current_phase: TrialPhase::Practice,
            trial_start: now,
            cursor: now,
            pending: VecDeque::new(),
            completed: Vec::new(),
        }
    }

    pub fn begin_trial(&mut self, trial_index: usize, phase: TrialPhase) {
        let now = self.clock.now();
        self.current_trial = trial_index;
        self.current_phase = phase;
        self.trial_start = now;
        self.cursor = now;
        self.pending.clear();
        debug!("Trial {} started at {}", trial_index, now);
    }

    pub fn queue(&mut self, kind: EventKind, duration: Duration) {
        self.cursor = self.cursor + duration;

        let event = Event {
            id: self.next_id,
            trial_index: self.current_trial,
            phase: self.current_phase,
            kind,
            scheduled_at: self.cursor,
            actual_onset: None,
            actual_wall: None,
            frame: None,
        };

        self.next_id += 1;
        self.pending.push_back(event);
    }

    pub fn queue_at(&mut self, kind: EventKind, at: Instant) {
        let event = Event {
            id: self.next_id,
            trial_index: self.current_trial,
            phase: self.current_phase,
            kind,
            scheduled_at: at,
            actual_onset: None,
            actual_wall: None,
            frame: None,
        };
        self.next_id += 1;
        self.pending.push_back(event);

        if at > self.cursor {
            self.cursor = at;
        }
    }

    pub fn poll_due(&mut self) -> Option<Event> {
        let now = self.clock.now();
        if self.pending.front()?.scheduled_at <= now {
            self.pending.pop_front()
        } else {
            None
        }
    }

    /// Block until the next pending event is due, then return it.
    /// On WASM this just polls without sleeping (browser controls yielding).
    /// On native, this calls `Clock::sleep_until` with spin-wait precision.
    pub fn wait_for_next(&mut self) -> Option<Event> {
        let scheduled_at = self.pending.front()?.scheduled_at;
        self.clock.sleep_until(scheduled_at);
        self.pending.pop_front()
    }

    /// Mark an event as having fired now. Stamps timestamps and moves it
    /// to the completed list.
    pub fn mark_onset(&mut self, mut event: Event, frame: Option<FrameTimestamp>) {
        let now = self.clock.now();
        let wall = self.clock.to_wall_time(now);

        event.actual_onset = Some(now);
        event.actual_wall = Some(wall);
        event.frame = frame;

        if let Some(err) = event.timing_error() {
            if err.abs() > Duration::from_millis(2) {
                warn!(
                    "Timing error on event {} ({:?}): {} - check system load",
                    event.id, event.kind, err
                );
            }
        }

        self.completed.push(event);
    }

    pub fn completed_events(&self) -> &[Event] {
        &self.completed
    }

    pub fn drain_completed(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.completed)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn is_trial_complete(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn trial_start(&self) -> Instant {
        self.trial_start
    }

    pub fn cursor(&self) -> Instant {
        self.cursor
    }

    pub fn time_until_next(&self) -> Duration {
        match self.pending.front() {
            None => Duration::ZERO,
            Some(evt) => {
                let now = self.clock.now();
                let diff = evt.scheduled_at - now;
                if diff.is_negative() {
                    Duration::ZERO
                } else {
                    diff
                }
            }
        }
    }
}
