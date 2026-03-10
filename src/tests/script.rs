use crate::clock::Clock;
use crate::data::SessionHeader;
use crate::script::ScriptHost;

#[allow(dead_code)]
fn make_host(dir: &std::path::Path) -> ScriptHost {
    let clock = Clock::new();
    let header = SessionHeader::new("TEST", "test_script", "/test.lua", Some(1), clock.info());
    ScriptHost::new(clock, dir, header, Some(1)).expect("ScriptHost::new failed")
}

#[test]
fn clock_now_ms_returns_non_negative_number() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run("assert(Clock.now_ms() >= 0)").unwrap();
}

#[test]
fn clock_now_ms_advances() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local t1 = Clock.now_ms()
        Clock.sleep(5)
        local t2 = Clock.now_ms()
        assert(t2 > t1, 'clock did not advance: t1=' .. t1 .. ' t2=' .. t2)
    ",
    )
    .unwrap();
}

#[test]
fn clock_sleep_takes_approximately_correct_time() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local t1 = Clock.now_ms()
        Clock.sleep(20)
        local elapsed = Clock.now_ms() - t1
        assert(elapsed >= 15, 'sleep too short: ' .. elapsed .. 'ms')
        assert(elapsed < 200, 'sleep too long: ' .. elapsed .. 'ms')
    ",
    )
    .unwrap();
}

#[test]
fn rand_int_is_within_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        for _ = 1, 100 do
            local n = Rand.int(1, 6)
            assert(n >= 1 and n <= 6, 'out of range: ' .. n)
        end
    ",
    )
    .unwrap();
}

#[test]
fn rand_int_same_seed_same_sequence() {
    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();
    let clock = Clock::new();

    let h1 = ScriptHost::new(
        clock.clone(),
        dir1.path(),
        SessionHeader::new("T", "t", "/t.lua", Some(1), clock.info()),
        Some(1),
    )
    .unwrap();
    let h2 = ScriptHost::new(
        clock.clone(),
        dir2.path(),
        SessionHeader::new("T", "t", "/t.lua", Some(1), clock.info()),
        Some(1),
    )
    .unwrap();

    let script = "
        SEQ = {}
        for i = 1, 10 do SEQ[i] = Rand.int(1, 100) end
    ";
    h1.run(script).unwrap();
    h2.run(script).unwrap();

    let seq1: Vec<i64> = h1
        .lua()
        .globals()
        .get::<mlua::Table>("SEQ")
        .unwrap()
        .sequence_values::<i64>()
        .collect::<mlua::Result<_>>()
        .unwrap();
    let seq2: Vec<i64> = h2
        .lua()
        .globals()
        .get::<mlua::Table>("SEQ")
        .unwrap()
        .sequence_values::<i64>()
        .collect::<mlua::Result<_>>()
        .unwrap();

    assert_eq!(seq1, seq2, "same seed should produce same sequence");
}

#[test]
fn rand_float_is_within_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        for _ = 1, 100 do
            local f = Rand.float(0.0, 1.0)
            assert(f >= 0.0 and f < 1.0, 'out of range: ' .. f)
        end
    ",
    )
    .unwrap();
}

#[test]
fn rand_shuffle_returns_same_elements() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local original = {10, 20, 30, 40, 50}
        local shuffled = Rand.shuffle(original)
        assert(#shuffled == #original, 'length mismatch')
        local sum = 0
        for _, v in ipairs(shuffled) do sum = sum + v end
        assert(sum == 150, 'element sum mismatch: ' .. sum)
    ",
    )
    .unwrap();
}

#[test]
fn rand_choice_returns_element_from_table() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local items = {'a', 'b', 'c', 'd'}
        for _ = 1, 20 do
            local c = Rand.choice(items)
            local found = false
            for _, v in ipairs(items) do
                if v == c then found = true end
            end
            assert(found, 'choice not in table: ' .. tostring(c))
        end
    ",
    )
    .unwrap();
}

#[test]
fn rand_balanced_shuffle_correct_length() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local conditions = {'A', 'B', 'C', 'D'}
        local trials = Rand.balanced_shuffle(conditions, 20)
        assert(#trials == 20, 'expected 20 trials, got ' .. #trials)
    ",
    )
    .unwrap();
}

#[test]
fn rand_balanced_shuffle_each_item_appears_evenly() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local items = {'A', 'B', 'C', 'D'}
        local trials = Rand.balanced_shuffle(items, 20)
        local counts = {}
        for _, v in ipairs(trials) do
            counts[v] = (counts[v] or 0) + 1
        end
        for _, item in ipairs(items) do
            local c = counts[item] or 0
            assert(c == 5, item .. ' appeared ' .. c .. ' times, expected 5')
        end
    ",
    )
    .unwrap();
}

#[test]
fn blank_primitive_sleeps() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run("_psychlib_blank(5)").unwrap();
}

#[test]
fn wait_key_primitive_times_out_and_returns_nil() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local r = _psychlib_wait_key({ timeout = 5 })
        assert(r == nil, 'expected nil on timeout, got: ' .. tostring(r))
    ",
    )
    .unwrap();
}

#[test]
fn ctx_is_injected_as_table() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run("assert(type(ctx) == 'table', 'ctx should be a table')")
        .unwrap();
}

#[test]
fn sequence_runs_nodes_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local log = {}
        local function make_log_node(label)
            return { run = function() table.insert(log, label) end }
        end
        local seq = Sequence({ make_log_node('a'), make_log_node('b'), make_log_node('c') })
        seq:run()
        assert(log[1] == 'a' and log[2] == 'b' and log[3] == 'c',
            'order wrong: ' .. table.concat(log, ','))
    ",
    )
    .unwrap();
}

#[test]
fn for_blocks_iterates_correct_count_and_sets_ctx_block() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.block = 0
        local blocks_seen = {}
        ForBlocks(3, function(block)
            return { run = function()
                table.insert(blocks_seen, ctx.block)
            end }
        end):run()
        assert(#blocks_seen == 3, 'expected 3 blocks, got ' .. #blocks_seen)
        assert(blocks_seen[1] == 1, 'block 1 wrong: ' .. blocks_seen[1])
        assert(blocks_seen[3] == 3, 'block 3 wrong: ' .. blocks_seen[3])
    ",
    )
    .unwrap();
}

#[test]
fn for_trials_iterates_list_and_sets_ctx_trial() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.trial_index = 0
        local seen = {}
        local trials = { {id=1}, {id=2}, {id=3} }
        ForTrials(trials, function(trial)
            return { run = function()
                table.insert(seen, ctx.trial.id)
            end }
        end):run()
        assert(#seen == 3, 'expected 3 trials')
        assert(seen[1] == 1 and seen[2] == 2 and seen[3] == 3,
            'trial ids wrong')
    ",
    )
    .unwrap();
}

#[test]
fn for_trials_increments_trial_index() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.trial_index = 0
        local indices = {}
        ForTrials({ {}, {}, {} }, function(_)
            return { run = function()
                table.insert(indices, ctx.trial_index)
            end }
        end):run()
        assert(indices[1] == 1, 'first trial_index should be 1')
        assert(indices[3] == 3, 'third trial_index should be 3')
    ",
    )
    .unwrap();
}

#[test]
fn if_node_runs_true_branch() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local ran = nil
        If(function() return true end,
            { run = function() ran = 'true_branch' end },
            { run = function() ran = 'false_branch' end }
        ):run()
        assert(ran == 'true_branch', 'expected true branch, got: ' .. tostring(ran))
    ",
    )
    .unwrap();
}

#[test]
fn if_node_runs_false_branch() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local ran = nil
        If(function() return false end,
            { run = function() ran = 'true_branch' end },
            { run = function() ran = 'false_branch' end }
        ):run()
        assert(ran == 'false_branch', 'expected false branch, got: ' .. tostring(ran))
    ",
    )
    .unwrap();
}

#[test]
fn if_node_no_else_does_nothing_on_false() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local ran = false
        If(function() return false end,
            { run = function() ran = true end }
        ):run()
        assert(not ran, 'should not have run')
    ",
    )
    .unwrap();
}

#[test]
fn loop_node_runs_correct_iterations() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local count = 0
        local i = 0
        Loop(function() i = i + 1; return i <= 5 end,
            { run = function() count = count + 1 end }
        ):run()
        assert(count == 5, 'expected 5 iterations, got ' .. count)
    ",
    )
    .unwrap();
}

#[test]
fn shuffle_returns_same_elements_different_order_possible() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local original = {1, 2, 3, 4, 5}
        local shuffled = Shuffle(original)
        assert(#shuffled == #original, 'length mismatch')
        local sum = 0
        for _, v in ipairs(shuffled) do sum = sum + v end
        assert(sum == 15, 'element sum wrong: ' .. sum)
        -- Original should be unmodified
        assert(original[1] == 1, 'original was mutated')
    ",
    )
    .unwrap();
}

#[test]
fn record_node_writes_to_csv() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.trial_index = 1
        ctx.block       = 1
        ctx.last_response = { key='left', rt_ms=312.4, timed_out=false, correct=true }
        Record({ condition='congruent' }):run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let csv = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |x| x == "csv"))
        .expect("no CSV file found");

    let contents = std::fs::read_to_string(csv.path()).unwrap();
    assert!(contents.contains("# participant: TEST"));
    assert!(contents.contains("congruent"));
    assert!(contents.contains("312"));
}

#[test]
fn record_node_omits_response_fields_when_no_last_response() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.trial_index   = 1
        ctx.block         = 1
        ctx.last_response = nil
        Record({ condition='baseline' }):run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let csv = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |x| x == "csv"))
        .expect("no CSV file found");

    let contents = std::fs::read_to_string(csv.path()).unwrap();
    assert!(
        !contents.contains("timed_out"),
        "timed_out should be absent"
    );
    assert!(!contents.contains("rt_ms"), "rt_ms should be absent");
    assert!(contents.contains("baseline"));
}

#[test]
fn timeline_run_initialises_ctx_fields() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local experiment = Timeline()
        experiment:add({ run = function()
            assert(ctx.trial_index == 0, 'trial_index not 0')
            assert(ctx.block == 0, 'block not 0')
            assert(ctx.trial == nil, 'trial not nil')
            assert(ctx.last_response == nil, 'last_response not nil')
        end })
        experiment:run()
    ",
    )
    .unwrap();
}

#[test]
fn sandbox_blocks_io_access() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    let result = host.run("local f = io.open('/etc/passwd', 'r')");
    assert!(result.is_err(), "sandbox should block io.open");
}

#[test]
fn sandbox_blocks_os_execute() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    let result = host.run("os.execute('ls')");
    assert!(result.is_err(), "sandbox should block os.execute");
}

#[test]
fn lua_value_to_json_object() {
    use crate::script::api_data::lua_value_to_json;
    let lua = mlua::Lua::new();
    let tbl: mlua::Table = lua
        .load(
            r#"
        return { condition = "congruent", correct = true, rt = 423.5 }
    "#,
        )
        .eval()
        .unwrap();
    let json = lua_value_to_json(mlua::Value::Table(tbl)).unwrap();
    assert_eq!(json["condition"], "congruent");
    assert_eq!(json["correct"], true);
    assert!((json["rt"].as_f64().unwrap() - 423.5).abs() < 0.001);
}

#[test]
fn lua_value_to_json_nested() {
    use crate::script::api_data::lua_value_to_json;
    let lua = mlua::Lua::new();
    let tbl: mlua::Table = lua
        .load(
            r#"
        return { outer = { inner = "value" } }
    "#,
        )
        .eval()
        .unwrap();
    let json = lua_value_to_json(mlua::Value::Table(tbl)).unwrap();
    assert_eq!(json["outer"]["inner"], "value");
}

#[test]
fn set_format_json_writes_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local experiment = Timeline()
        experiment:set_format('json')
        experiment:add({ run = function()
            ctx.trial_index   = 1
            ctx.block         = 1
            ctx.last_response = { key='right', rt_ms=250.0, timed_out=false, correct=true }
            Record({ condition='congruent' }):run()
        end })
        experiment:run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let json = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |x| x == "json"))
        .expect("no JSON file found");

    let contents = std::fs::read_to_string(json.path()).unwrap();
    let root: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(root["session"]["participant"], "TEST");
    let trials = root["trials"].as_array().unwrap();
    assert_eq!(trials.len(), 1);
    assert_eq!(trials[0]["condition"], "congruent");
    assert_eq!(trials[0]["correct"], true);
}

#[test]
fn set_format_json_does_not_write_csv() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local experiment = Timeline()
        experiment:set_format('json')
        experiment:add({ run = function()
            ctx.trial_index = 1
            ctx.block = 1
            ctx.last_response = nil
            Record({ x = 1 }):run()
        end })
        experiment:run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let has_csv = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().map_or(false, |x| x == "csv"));

    assert!(!has_csv, "JSON format should not produce a CSV file");
}

#[test]
fn set_format_both_writes_csv_and_json() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local experiment = Timeline()
        experiment:set_format('both')
        experiment:add({ run = function()
            ctx.trial_index   = 1
            ctx.block         = 1
            ctx.last_response = { key='left', rt_ms=310.0, timed_out=false, correct=false }
            Record({ condition='incongruent' }):run()
        end })
        experiment:run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let has_csv = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().map_or(false, |x| x == "csv"));
    let has_json = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().map_or(false, |x| x == "json"));

    assert!(has_csv, "both format should produce a CSV file");
    assert!(has_json, "both format should produce a JSON file");
}

#[test]
fn set_format_default_is_csv() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        ctx.trial_index = 1
        ctx.block = 1
        ctx.last_response = nil
        Record({ x = 1 }):run()
        _psychlib_save()
    ",
    )
    .unwrap();

    let has_csv = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().map_or(false, |x| x == "csv"));

    assert!(has_csv, "default format should produce a CSV file");
}

#[test]
fn set_format_unknown_string_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    let result = host.run(
        "
        local experiment = Timeline()
        experiment:set_format('parquet')
    ",
    );
    assert!(result.is_err(), "unknown format should return an error");
}
