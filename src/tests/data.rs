use crate::clock::{Clock, Instant};
use crate::data::record::TrialRecord;
use crate::data::session::SessionHeader;
#[allow(unused_imports)]
use crate::data::writer::{CsvWriter, DataStore, JsonWriter, data_filename};
use crate::io::keyboard::KeyCode;
use crate::io::response::Response;
use crate::scheduler::TrialPhase;

#[allow(dead_code)]
fn make_header() -> SessionHeader {
    let clock = Clock::new();
    SessionHeader::new(
        "P001",
        "flanker_task",
        "/experiments/flanker_task.lua",
        Some(42),
        clock.info(),
    )
}

#[allow(dead_code)]
fn make_response(rt_ms: f64) -> Response {
    let onset = Instant::from_nanos(0);
    let timestamp = Instant::from_nanos((rt_ms * 1_000_000.0) as u64);
    let rt = timestamp - onset;
    Response {
        key: KeyCode::F,
        rt,
        timestamp,
        onset,
    }
}

#[allow(dead_code)]
fn make_record(trial_index: usize, response: Option<Response>) -> TrialRecord {
    let clock = Clock::new();
    let now = clock.now();
    let wall = clock.to_wall_time(now);
    TrialRecord::new(
        trial_index,
        0,
        TrialPhase::Experiment,
        response,
        &[],
        serde_json::json!({ "condition": "congruent", "correct": true }),
        now,
        wall,
    )
}

#[test]
fn session_header_fields_are_set() {
    let h = make_header();
    assert_eq!(h.participant, "P001");
    assert_eq!(h.script_name, "flanker_task");
    assert_eq!(h.seed, Some(42));
    assert!(h.ended_at.is_none());
}

#[test]
fn session_header_close_sets_ended_at() {
    let mut h = make_header();
    assert!(h.ended_at.is_none());
    h.close();
    assert!(h.ended_at.is_some());
    assert!(h.duration().is_some());
}

#[test]
fn session_header_csv_comments_contain_key_fields() {
    let h = make_header();
    let comments = h.to_csv_comments();
    assert!(comments.contains("# participant: P001"));
    assert!(comments.contains("# script_name: flanker_task"));
    assert!(comments.contains("# seed: 42"));
    assert!(comments.starts_with("# psychlib_version:"));
}

#[test]
fn session_header_csv_comments_all_lines_start_with_hash() {
    let h = make_header();
    for line in h.to_csv_comments().lines() {
        assert!(line.starts_with('#'), "Line missing # prefix: {:?}", line);
    }
}

#[test]
fn session_header_extra_metadata() {
    let h = make_header()
        .with_extra("lab", "cog-neuro-b")
        .with_extra("irb", "2024-001");
    let comments = h.to_csv_comments();
    assert!(comments.contains("# lab: cog-neuro-b"));
    assert!(comments.contains("# irb: 2024-001"));
}

#[test]
fn session_header_entropy_seed_label() {
    let clock = Clock::new();
    let h = SessionHeader::new("P002", "exp", "/exp.lua", None, clock.info());
    assert!(h.to_csv_comments().contains("# seed: entropy"));
}

#[test]
fn data_filename_format() {
    let h = make_header();
    let name = data_filename(&h, "csv");
    assert!(name.starts_with("P001_flanker_task_"));
    assert!(name.ends_with(".csv"));
}

#[test]
fn data_filename_sanitises_special_chars() {
    let clock = Clock::new();
    let h = SessionHeader::new("P 001/bad", "my experiment!", "/e.lua", None, clock.info());
    let name = data_filename(&h, "csv");
    assert!(!name.contains(' '));
    assert!(!name.contains('/'));
    assert!(!name.contains('!'));
}

#[test]
fn trial_record_with_response() {
    let r = make_record(0, Some(make_response(423.5)));
    assert_eq!(r.response_key, Some("f".to_string()));
    assert!((r.rt_ms.unwrap() - 423.5).abs() < 0.01);
    assert!(!r.timed_out);
}

#[test]
fn trial_record_timeout() {
    let r = make_record(1, None);
    assert_eq!(r.response_key, None);
    assert_eq!(r.rt_ms, None);
    assert!(r.timed_out);
}

#[test]
fn trial_record_custom_fields() {
    let r = make_record(0, None);
    assert_eq!(r.custom["condition"], "congruent");
    assert_eq!(r.custom["correct"], true);
}

#[test]
fn trial_record_csv_row_has_correct_length() {
    let r = make_record(0, Some(make_response(300.0)));
    let keys = ["condition", "correct"];
    let cols = TrialRecord::csv_column_names(&keys);
    let row = r.to_csv_row(&keys);
    assert_eq!(
        cols.len(),
        row.len(),
        "Column count mismatch: headers={:?} row={:?}",
        cols,
        row
    );
}

#[test]
fn trial_record_csv_row_values() {
    let r = make_record(0, Some(make_response(350.0)));
    let row = r.to_csv_row(&["condition", "correct"]);

    assert_eq!(row[0], "0");
    let timed_out_col = TrialRecord::csv_column_names(&["condition", "correct"])
        .iter()
        .position(|c| c == "timed_out")
        .unwrap();
    assert_eq!(row[timed_out_col], "false");
}

#[test]
fn trial_record_csv_columns_are_sorted_custom_last() {
    let cols = TrialRecord::csv_column_names(&["zzz_last", "aaa_first"]);
    assert_eq!(cols[0], "trial_index");
    let custom_start = cols.iter().position(|c| c == "aaa_first").unwrap();
    let custom_end = cols.iter().position(|c| c == "zzz_last").unwrap();
    assert!(custom_start < custom_end, "Custom columns should be sorted");
}

#[test]
fn csv_writer_creates_file_and_writes_trials() {
    let dir = tempfile::tempdir().unwrap();
    let h = make_header();

    let mut writer = CsvWriter::create(dir.path(), h).unwrap();

    writer
        .write_trial(&make_record(0, Some(make_response(400.0))))
        .unwrap();
    writer.write_trial(&make_record(1, None)).unwrap();

    let path = writer.path().to_owned();
    Box::new(writer).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();

    assert!(contents.contains("# participant: P001"));
    assert!(contents.contains("trial_index"));
    let data_lines: Vec<_> = contents
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("trial_index"))
        .collect();
    assert_eq!(data_lines.len(), 2);
}

#[test]
fn csv_writer_timeout_row_has_empty_rt() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = CsvWriter::create(dir.path(), make_header()).unwrap();
    w.write_trial(&make_record(0, None)).unwrap();
    let path = w.path().to_owned();
    Box::new(w).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("true"));
}

#[test]
fn json_writer_creates_valid_ndjson() {
    let dir = tempfile::tempdir().unwrap();
    let h = make_header();

    let mut writer = JsonWriter::create(dir.path(), h).unwrap();
    writer
        .write_trial(&make_record(0, Some(make_response(300.0))))
        .unwrap();
    writer.write_trial(&make_record(1, None)).unwrap();

    let path = writer.path().to_owned();
    Box::new(writer).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<_> = contents.lines().filter(|l| !l.trim().is_empty()).collect();

    for line in &lines {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("Invalid JSON line: {}\nError: {}", line, e));
    }

    let session: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(session["type"], "session");
    assert_eq!(session["participant"], "P001");

    let trial: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(trial["type"], "trial");
    assert_eq!(trial["trial_index"], 0);
}

#[test]
fn json_writer_file_extension_is_ndjson() {
    let dir = tempfile::tempdir().unwrap();
    let w = JsonWriter::create(dir.path(), make_header()).unwrap();
    assert!(w.path().to_string_lossy().ends_with(".ndjson"));
    Box::new(w).close().unwrap();
}
