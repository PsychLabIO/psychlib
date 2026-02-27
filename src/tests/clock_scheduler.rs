#[allow(unused_imports)]
use crate::clock::{Clock, Duration, Instant};
#[allow(unused_imports)]
use crate::scheduler::{EventKind, Scheduler, TrialPhase};

#[test]
fn duration_constructors_are_consistent() {
    assert_eq!(Duration::from_millis(1).as_nanos(), 1_000_000);
    assert_eq!(Duration::from_micros(1000).as_nanos(), 1_000_000);
    assert_eq!(Duration::from_secs(0.001).as_nanos(), 1_000_000);
}

#[test]
fn duration_display_picks_right_unit() {
    assert!(Duration::from_nanos(500).to_string().contains("ns"));
    assert!(Duration::from_micros(500).to_string().contains("µs"));
    assert!(Duration::from_millis(500).to_string().contains("ms"));
    assert!(Duration::from_secs(1.5).to_string().contains('s'));
}

#[test]
fn duration_signed_arithmetic() {
    let a = Duration::from_millis(10);
    let b = Duration::from_millis(15);
    let diff = a - b;
    assert!(diff.is_negative());
    assert_eq!(diff.as_millis(), -5);
    assert_eq!(diff.abs().as_millis(), 5);
}

#[test]
fn duration_to_std_panics_on_negative() {
    let result = std::panic::catch_unwind(|| Duration::from_millis(-1).to_std());
    assert!(result.is_err());
}

#[test]
fn duration_checked_to_std_returns_none_on_negative() {
    assert!(Duration::from_millis(-1).checked_to_std().is_none());
    assert!(Duration::from_millis(1).checked_to_std().is_some());
}

#[test]
fn instant_subtraction_is_signed() {
    let a = Instant::from_nanos(1000);
    let b = Instant::from_nanos(1500);
    let diff = a - b;
    assert!(diff.is_negative());
    assert_eq!(diff.as_nanos(), -500);
}

#[test]
fn instant_add_duration() {
    let t = Instant::from_nanos(1_000_000);
    let d = Duration::from_millis(5);
    let result = t + d;
    assert_eq!(result.as_nanos(), 6_000_000);
}

#[test]
fn instant_sub_duration_saturates_at_zero() {
    let t = Instant::from_nanos(100);
    let large = Duration::from_millis(999);
    let result = t - large;
    assert_eq!(result, Instant::ZERO);
}

#[test]
fn clock_now_is_monotonic() {
    let clock = Clock::new();
    let t1 = clock.now();
    for _ in 0..10_000 {
        std::hint::spin_loop();
    }
    let t2 = clock.now();
    assert!(t2 > t1, "clock must be monotonic");
}

#[test]
fn clock_elapsed_is_non_negative() {
    let clock = Clock::new();
    let start = clock.now();
    let elapsed = clock.elapsed(start);
    assert!(!elapsed.is_negative());
}

#[test]
fn clock_sleep_accuracy_within_5ms() {
    let clock = Clock::new();
    let target = Duration::from_millis(50);
    let before = clock.now();
    clock.sleep(target);
    let actual = clock.elapsed(before);

    let error = (actual - target).abs();
    assert!(
        error < Duration::from_millis(5),
        "Sleep error too large: {} (target=50ms, actual={})",
        error,
        actual
    );
}

#[test]
fn clock_frame_log_records_and_clears() {
    let clock = Clock::new();
    clock.record_frame("fixation:trial_0");
    clock.record_frame("stimulus:trial_0");

    let log = clock.frame_log();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].label, "fixation:trial_0");

    clock.clear_frame_log();
    assert!(clock.frame_log().is_empty());
}

#[test]
fn clock_wall_time_advances_with_instant() {
    let clock = Clock::new();
    let t0 = clock.now();
    for _ in 0..1_000_000 {
        std::hint::spin_loop();
    }
    let t1 = clock.now();

    let w0 = clock.to_wall_time(t0);
    let w1 = clock.to_wall_time(t1);
    assert!(w1 > w0);
}

#[test]
fn scheduler_events_fire_in_order() {
    let clock = Clock::new();
    let mut sched = Scheduler::new(clock);

    sched.begin_trial(0, TrialPhase::Experiment);
    sched.queue(EventKind::Fixation, Duration::from_millis(10));
    sched.queue(EventKind::StimulusOnset, Duration::from_millis(10));
    sched.queue(EventKind::Iti, Duration::from_millis(10));

    assert_eq!(sched.pending_count(), 3);

    let mut kinds = vec![];
    while let Some(evt) = sched.wait_for_next() {
        let kind = evt.kind.clone();
        sched.mark_onset(evt, None);
        kinds.push(kind);
    }

    assert_eq!(
        kinds,
        vec![
            EventKind::Fixation,
            EventKind::StimulusOnset,
            EventKind::Iti,
        ]
    );
    assert!(sched.is_trial_complete());
}

#[test]
fn scheduler_timing_errors_are_small() {
    let clock = Clock::new();
    let mut sched = Scheduler::new(clock);

    sched.begin_trial(0, TrialPhase::Experiment);
    for _ in 0..3 {
        sched.queue(EventKind::StimulusOnset, Duration::from_millis(20));
    }

    while let Some(evt) = sched.wait_for_next() {
        sched.mark_onset(evt, None);
    }

    for evt in sched.completed_events() {
        let err = evt.timing_error().unwrap().abs();
        assert!(
            err < Duration::from_millis(5),
            "Event {:?} timing error too large: {}",
            evt.kind,
            err
        );
    }
}

#[test]
fn scheduler_begin_trial_resets_pending() {
    let clock = Clock::new();
    let mut sched = Scheduler::new(clock);

    sched.begin_trial(0, TrialPhase::Experiment);
    sched.queue(EventKind::Fixation, Duration::from_millis(100));
    assert_eq!(sched.pending_count(), 1);

    sched.begin_trial(1, TrialPhase::Experiment);
    assert_eq!(sched.pending_count(), 0);
}

#[test]
fn scheduler_drain_completed_is_empty_after_drain() {
    let clock = Clock::new();
    let mut sched = Scheduler::new(clock);

    sched.begin_trial(0, TrialPhase::Experiment);
    sched.queue(EventKind::Blank, Duration::ZERO);

    let evt = sched.wait_for_next().unwrap();
    sched.mark_onset(evt, None);

    let drained = sched.drain_completed();
    assert_eq!(drained.len(), 1);
    assert!(sched.completed_events().is_empty());
}
