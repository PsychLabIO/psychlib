#[allow(unused_imports)]
use crate::clock::{Clock, Duration, Instant};
#[allow(unused_imports)]
use crate::io::keyboard::{KeyCode, KeyState};
#[allow(unused_imports)]
use crate::io::response::{Response, ResponseOutcome, ResponseWindow};

#[test]
fn keycode_from_name_letters() {
    assert_eq!(KeyCode::from_name("f"), Some(KeyCode::F));
    assert_eq!(KeyCode::from_name("F"), Some(KeyCode::F)); // case-insensitive
    assert_eq!(KeyCode::from_name("j"), Some(KeyCode::J));
    assert_eq!(KeyCode::from_name("z"), Some(KeyCode::Z));
}

#[test]
fn keycode_from_name_digits() {
    assert_eq!(KeyCode::from_name("0"), Some(KeyCode::Key0));
    assert_eq!(KeyCode::from_name("9"), Some(KeyCode::Key9));
}

#[test]
fn keycode_from_name_special() {
    assert_eq!(KeyCode::from_name("space"), Some(KeyCode::Space));
    assert_eq!(KeyCode::from_name(" "), Some(KeyCode::Space));
    assert_eq!(KeyCode::from_name("return"), Some(KeyCode::Return));
    assert_eq!(KeyCode::from_name("enter"), Some(KeyCode::Return));
    assert_eq!(KeyCode::from_name("escape"), Some(KeyCode::Escape));
    assert_eq!(KeyCode::from_name("esc"), Some(KeyCode::Escape));
    assert_eq!(KeyCode::from_name("left"), Some(KeyCode::LeftArrow));
    assert_eq!(KeyCode::from_name("right"), Some(KeyCode::RightArrow));
}

#[test]
fn keycode_from_name_function_keys() {
    assert_eq!(KeyCode::from_name("f1"), Some(KeyCode::F1));
    assert_eq!(KeyCode::from_name("f12"), Some(KeyCode::F12));
}

#[test]
fn keycode_from_name_unknown_returns_none() {
    assert_eq!(KeyCode::from_name("banana"), None);
    assert_eq!(KeyCode::from_name(""), None);
}

#[test]
fn keycode_as_name_round_trips() {
    let keys = [
        KeyCode::A,
        KeyCode::Z,
        KeyCode::F,
        KeyCode::J,
        KeyCode::Space,
        KeyCode::Return,
        KeyCode::Escape,
        KeyCode::LeftArrow,
        KeyCode::RightArrow,
        KeyCode::F1,
        KeyCode::F12,
    ];
    for key in keys {
        let name = key.as_name();
        let parsed = KeyCode::from_name(name);
        assert_eq!(
            parsed,
            Some(key),
            "round-trip failed for {:?} (name={})",
            key,
            name
        );
    }
}

#[test]
fn keycode_display() {
    assert_eq!(KeyCode::F.to_string(), "f");
    assert_eq!(KeyCode::Space.to_string(), "space");
    assert_eq!(KeyCode::LeftArrow.to_string(), "left");
}

#[test]
fn keycode_modifiers_are_detected() {
    assert!(KeyCode::LeftShift.is_modifier());
    assert!(KeyCode::RightCtrl.is_modifier());
    assert!(KeyCode::LeftAlt.is_modifier());
    assert!(!KeyCode::F.is_modifier());
    assert!(!KeyCode::Space.is_modifier());
}

#[test]
fn response_rt_ms_is_correct() {
    let onset = Instant::from_nanos(0);
    let timestamp = Instant::from_nanos(423_000_000);
    let rt = timestamp - onset;

    let resp = Response {
        key: KeyCode::F,
        rt,
        timestamp,
        onset,
    };

    let rt_ms = resp.rt_ms();
    assert!(
        (rt_ms - 423.0).abs() < 0.001,
        "Expected ~423ms, got {}",
        rt_ms
    );
}

#[test]
fn response_display_contains_key_and_rt() {
    let onset = Instant::from_nanos(0);
    let timestamp = Instant::from_nanos(350_000_000);
    let rt = timestamp - onset;

    let resp = Response {
        key: KeyCode::J,
        rt,
        timestamp,
        onset,
    };
    let s = resp.to_string();
    assert!(s.contains("j"), "Display missing key: {}", s);
    assert!(s.contains("350"), "Display missing RT:  {}", s);
}

#[test]
fn response_outcome_predicates() {
    let onset = Instant::from_nanos(0);
    let timestamp = Instant::from_nanos(100_000_000);
    let rt = timestamp - onset;

    let r = Response {
        key: KeyCode::F,
        rt,
        timestamp,
        onset,
    };
    let hit = ResponseOutcome::Response(r);
    let timeout = ResponseOutcome::Timeout;

    assert!(hit.is_response());
    assert!(!hit.is_timeout());
    assert!(timeout.is_timeout());
    assert!(!timeout.is_response());
}

#[test]
fn response_outcome_unwrap() {
    let onset = Instant::from_nanos(0);
    let timestamp = Instant::from_nanos(100_000_000);
    let rt = timestamp - onset;

    let r = Response {
        key: KeyCode::J,
        rt,
        timestamp,
        onset,
    };
    let outcome = ResponseOutcome::Response(r);

    assert!(outcome.response().is_some());
    assert!(ResponseOutcome::Timeout.response().is_none());
}

#[test]
fn response_window_timeout_fires() {
    let clock = Clock::new();
    let mut window = ResponseWindow::new(&clock)
        .accept_keys(&[KeyCode::F, KeyCode::J])
        .timeout(Duration::from_millis(10));

    window.arm();
    let outcome = window.wait();

    assert!(outcome.is_timeout(), "Expected timeout, got {:?}", outcome);
}

#[test]
fn response_window_arm_without_flush_does_not_panic() {
    let clock = Clock::new();
    let mut window = ResponseWindow::new(&clock)
        .flush_on_arm(false)
        .timeout(Duration::from_millis(1));
    window.arm();
    let _ = window.wait();
}

#[test]
fn response_window_arm_at_uses_provided_onset() {
    let clock = Clock::new();
    let onset = clock.now();

    let mut window = ResponseWindow::new(&clock).timeout(Duration::from_millis(1));
    window.arm_at(onset);
    let _ = window.wait();
}

#[test]
#[should_panic(expected = "arm()")]
fn response_window_wait_without_arm_panics() {
    let clock = Clock::new();
    let window = ResponseWindow::new(&clock).timeout(Duration::from_millis(1));
    window.wait();
}
