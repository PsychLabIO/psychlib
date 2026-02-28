use crate::renderer::RenderConfig;
#[allow(unused_imports)]
use crate::runtime::{ExperimentConfig, headless_run};

#[allow(dead_code)]
fn config(dir: &std::path::Path, script: &str, seed: u64) -> ExperimentConfig {
    let script_path = dir.join("experiment.lua");
    std::fs::write(&script_path, script).unwrap();

    ExperimentConfig {
        participant: "TEST".into(),
        script_path,
        output_dir: dir.join("data"),
        seed: Some(seed),
        render: RenderConfig::default(),
    }
}

#[allow(dead_code)]
fn find_csv(data_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(data_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |x| x == "csv"))
        .map(|e| e.path())
}

#[test]
fn headless_minimal_script_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        Data.record({ condition = "test", value = 42 })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).expect("headless_run should succeed");
}

#[test]
fn headless_creates_csv_with_data() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        for i = 1, 5 do
            Data.record({ trial = i, condition = "x", correct = true })
        end
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();

    let csv = find_csv(&dir.path().join("data")).expect("CSV not found");
    let contents = std::fs::read_to_string(csv).unwrap();

    assert!(contents.contains("# participant: TEST"));
    assert!(contents.contains("# seed: 1"));
    assert!(contents.contains("trial_index"));

    let data_rows: Vec<_> = contents
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("trial_index"))
        .collect();
    assert_eq!(data_rows.len(), 5, "Expected 5 rows, got: {:?}", data_rows);
}

#[test]
fn headless_trial_index_increments_across_records() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        Data.record({ x = "a" })
        Data.record({ x = "b" })
        Data.record({ x = "c" })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();

    let csv = find_csv(&dir.path().join("data")).unwrap();
    let contents = std::fs::read_to_string(csv).unwrap();

    let data_rows: Vec<_> = contents
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("trial_index"))
        .collect();

    assert_eq!(data_rows.len(), 3);

    let indices: Vec<&str> = data_rows
        .iter()
        .map(|row| row.split(',').next().unwrap_or(""))
        .collect();

    assert_eq!(
        indices,
        vec!["0", "1", "2"],
        "Trial indices should be 0,1,2 — got {:?}",
        indices
    );
}

#[test]
fn headless_clock_sleep_works() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        local t0 = Clock.now_ms()
        Clock.sleep(20)
        local elapsed = Clock.now_ms() - t0
        assert(elapsed >= 15, "sleep too short: " .. elapsed .. "ms")
        assert(elapsed < 500, "sleep too long: " .. elapsed .. "ms")
        Data.record({ elapsed_ms = elapsed })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();
}

#[test]
fn headless_trial_blank_does_not_panic() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        Trial.blank(5)
        Trial.blank(0)
        Data.record({ done = true })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();
}

#[test]
fn headless_wait_key_times_out_returns_nil() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        local r = Trial.wait_key({ keys = {"f", "j"}, timeout = 5 })
        local timed_out = (r == nil)
        Data.record({ timed_out = timed_out })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();

    let csv = find_csv(&dir.path().join("data")).unwrap();
    let contents = std::fs::read_to_string(csv).unwrap();
    assert!(contents.contains("true"), "timed_out should be true in CSV");
}

#[test]
fn headless_rand_seed_is_reproducible() {
    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();

    let script = r#"
        local vals = {}
        for i = 1, 10 do
            vals[i] = Rand.int(1, 100)
        end
        Data.record({ v1=vals[1], v2=vals[2], v3=vals[3] })
        Data.save()
    "#;

    headless_run(config(dir1.path(), script, 42)).unwrap();
    headless_run(config(dir2.path(), script, 42)).unwrap();

    let csv1 = std::fs::read_to_string(find_csv(&dir1.path().join("data")).unwrap()).unwrap();
    let csv2 = std::fs::read_to_string(find_csv(&dir2.path().join("data")).unwrap()).unwrap();

    let rows1: Vec<_> = csv1.lines().filter(|l| !l.starts_with('#')).collect();
    let rows2: Vec<_> = csv2.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(rows1, rows2, "Same seed should produce identical data rows");
}

#[test]
fn headless_balanced_shuffle_produces_correct_count() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        local conds = {"A", "B", "C", "D"}
        local trials = Rand.balanced_shuffle(conds, 20)
        assert(#trials == 20, "Expected 20 trials, got " .. #trials)

        -- Count occurrences
        local counts = {}
        for _, v in ipairs(trials) do
            counts[v] = (counts[v] or 0) + 1
        end
        for _, c in ipairs(conds) do
            assert(counts[c] == 5, c .. " count: " .. (counts[c] or 0))
        end

        Data.record({ n_trials = #trials })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();
}

#[test]
fn headless_lua_runtime_error_propagates() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        error("intentional test error")
    "#,
        1,
    );

    let result = headless_run(cfg);
    assert!(result.is_err(), "Expected error from Lua error()");
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.contains("intentional test error"),
        "Error message should contain Lua error text: {}",
        msg
    );
}

#[test]
fn headless_sandbox_blocks_io() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        local f = io.open("/etc/passwd", "r")
    "#,
        1,
    );

    let result = headless_run(cfg);
    assert!(result.is_err(), "Sandbox should block io.open");
}

#[test]
fn headless_csv_has_comment_header_then_column_row_then_data() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        Data.record({ condition = "congruent", correct = true, rt = 350.0 })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();

    let csv = find_csv(&dir.path().join("data")).unwrap();
    let contents = std::fs::read_to_string(csv).unwrap();
    let lines: Vec<&str> = contents.lines().collect();

    let first_non_comment = lines.iter().position(|l| !l.starts_with('#')).unwrap();
    for i in 0..first_non_comment {
        assert!(lines[i].starts_with('#'), "Line {} should be a comment", i);
    }

    let header_line = lines[first_non_comment];
    assert!(
        header_line.contains("trial_index"),
        "First non-comment line should be column header: {}",
        header_line
    );
    assert!(
        header_line.contains("condition"),
        "Column header should contain custom field 'condition': {}",
        header_line
    );

    let data_lines: Vec<_> = lines[first_non_comment + 1..]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(data_lines.len(), 1);
    assert!(data_lines[0].contains("congruent"));
}

#[test]
fn headless_multiple_blocks_tracked() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = config(
        dir.path(),
        r#"
        Trial.set_block(0)
        Data.record({ phase = "practice" })
        Trial.set_block(1)
        Data.record({ phase = "experiment" })
        Trial.set_block(1)
        Data.record({ phase = "experiment" })
        Data.save()
    "#,
        1,
    );

    headless_run(cfg).unwrap();

    let csv = find_csv(&dir.path().join("data")).unwrap();
    let contents = std::fs::read_to_string(csv).unwrap();

    let data_rows: Vec<_> = contents
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("trial_index"))
        .collect();

    assert_eq!(data_rows.len(), 3);

    assert!(
        data_rows[0].split(',').nth(1).unwrap_or("") == "0",
        "First row block_index should be 0: {}",
        data_rows[0]
    );

    assert!(
        data_rows[1].split(',').nth(1).unwrap_or("") == "1",
        "Second row block_index should be 1: {}",
        data_rows[1]
    );
}
