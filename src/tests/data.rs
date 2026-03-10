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

    let mut b = TrialRecord::builder()
        .field("trial_index", trial_index as u64)
        .field("block_index", 0u64)
        .field(
            "phase",
            format!("{:?}", TrialPhase::Experiment).to_lowercase(),
        )
        .field(
            "recorded_at_wall",
            wall.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        );

    b = match response {
        Some(ref r) => b
            .field("response_key", r.key.to_string())
            .field("rt_ms", r.rt_ms())
            .field("timed_out", false),
        None => b.field("timed_out", true),
    };

    b.field("condition", "congruent")
        .field("correct", true)
        .build()
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
fn session_header_extra_is_deterministic() {
    let h = make_header()
        .with_extra("zzz", "last")
        .with_extra("aaa", "first");
    let comments = h.to_csv_comments();
    let aaa_pos = comments.find("# aaa:").unwrap();
    let zzz_pos = comments.find("# zzz:").unwrap();
    assert!(
        aaa_pos < zzz_pos,
        "extra keys should appear in sorted order"
    );
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
fn trial_record_with_response_has_expected_fields() {
    let r = make_record(0, Some(make_response(423.5)));
    assert_eq!(
        r.fields.get("response_key").and_then(|v| v.as_str()),
        Some("f")
    );
    let rt = r.fields.get("rt_ms").and_then(|v| v.as_f64()).unwrap();
    assert!((rt - 423.5).abs() < 0.01, "rt_ms={}", rt);
    assert_eq!(
        r.fields.get("timed_out").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn trial_record_timeout_has_no_response_fields() {
    let r = make_record(1, None);
    assert!(
        r.fields.get("response_key").is_none(),
        "timed-out record should not have response_key"
    );
    assert!(
        r.fields.get("rt_ms").is_none(),
        "timed-out record should not have rt_ms"
    );
    assert_eq!(
        r.fields.get("timed_out").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn trial_record_custom_fields_are_present() {
    let r = make_record(0, None);
    assert_eq!(
        r.fields.get("condition").and_then(|v| v.as_str()),
        Some("congruent")
    );
    assert_eq!(
        r.fields.get("correct").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn trial_record_builder_field_opt_omits_none() {
    let r = TrialRecord::builder()
        .field("present", "yes")
        .field_opt("absent", Option::<String>::None)
        .build();
    assert!(r.fields.contains_key("present"));
    assert!(!r.fields.contains_key("absent"));
}

#[test]
fn trial_record_builder_merge_extends_fields() {
    let extra: Vec<(String, serde_json::Value)> = vec![
        ("a".to_string(), serde_json::json!(1)),
        ("b".to_string(), serde_json::json!(2)),
    ];

    let r = TrialRecord::builder()
        .field("existing", "yes")
        .merge(extra)
        .build();

    assert!(r.fields.contains_key("existing"));
    assert_eq!(r.fields.get("a").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(r.fields.get("b").and_then(|v| v.as_u64()), Some(2));
}

#[test]
fn trial_record_builder_merge_overwrites_on_collision() {
    let extra: Vec<(String, serde_json::Value)> =
        vec![("key".to_string(), serde_json::json!("new"))];

    let r = TrialRecord::builder()
        .field("key", "old")
        .merge(extra)
        .build();

    assert_eq!(r.fields.get("key").and_then(|v| v.as_str()), Some("new"));
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
fn csv_writer_column_order_matches_first_record() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = CsvWriter::create(dir.path(), make_header()).unwrap();
    w.write_trial(&make_record(0, Some(make_response(300.0))))
        .unwrap();
    let path = w.path().to_owned();
    Box::new(w).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let header_line = contents
        .lines()
        .find(|l| !l.starts_with('#') && !l.trim().is_empty())
        .unwrap();

    let cols: Vec<&str> = header_line.split(',').collect();
    assert_eq!(cols[0], "trial_index");
}

#[test]
fn csv_writer_timed_out_row_contains_true() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = CsvWriter::create(dir.path(), make_header()).unwrap();
    w.write_trial(&make_record(0, None)).unwrap();
    let path = w.path().to_owned();
    Box::new(w).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let data_line = contents
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("trial_index"))
        .next()
        .unwrap();

    assert!(data_line.contains("true"), "timed_out should be true");
}

#[test]
fn csv_writer_response_row_has_rt() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = CsvWriter::create(dir.path(), make_header()).unwrap();
    w.write_trial(&make_record(0, Some(make_response(350.0))))
        .unwrap();
    let path = w.path().to_owned();
    Box::new(w).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("350"), "expected rt value in CSV");
}

#[test]
fn json_writer_creates_valid_json() {
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
    let root: serde_json::Value =
        serde_json::from_str(&contents).unwrap_or_else(|e| panic!("Invalid JSON: {}", e));

    assert_eq!(root["session"]["participant"], "P001");

    let trials = root["trials"].as_array().unwrap();
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0]["trial_index"], 0);
    assert_eq!(trials[1]["trial_index"], 1);
}

#[test]
fn json_writer_timed_out_trial_has_no_rt_key() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = JsonWriter::create(dir.path(), make_header()).unwrap();
    w.write_trial(&make_record(0, None)).unwrap();
    let path = w.path().to_owned();
    Box::new(w).close().unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let root: serde_json::Value = serde_json::from_str(&contents).unwrap();
    let trial = &root["trials"][0];

    assert!(
        trial.get("rt_ms").is_none(),
        "timed-out trial should have no rt_ms key"
    );
    assert!(
        trial.get("response_key").is_none(),
        "timed-out trial should have no response_key"
    );
    assert_eq!(trial["timed_out"], true);
}

#[test]
fn json_writer_file_extension_is_json() {
    let dir = tempfile::tempdir().unwrap();
    let w = JsonWriter::create(dir.path(), make_header()).unwrap();
    assert!(w.path().to_string_lossy().ends_with(".json"));
    Box::new(w).close().unwrap();
}
