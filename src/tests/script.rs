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
        for i = 1, 10 do
            SEQ[i] = Rand.int(1, 100)
        end
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

    assert_eq!(seq1, seq2, "Same seed should produce same sequence");
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
        -- Check all elements present (by sum)
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
        -- Each of 4 items should appear exactly 5 times
        for _, item in ipairs(items) do
            local c = counts[item] or 0
            assert(c == 5, item .. ' appeared ' .. c .. ' times, expected 5')
        end
    ",
    )
    .unwrap();
}

#[test]
fn trial_blank_sleeps() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run("Trial.blank(5)").unwrap();
}

#[test]
fn trial_wait_key_times_out_and_returns_nil() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local r = Trial.wait_key({ timeout = 5 })
        assert(r == nil, 'expected nil on timeout, got: ' .. tostring(r))
    ",
    )
    .unwrap();
}

#[test]
fn trial_index_advances() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local i0 = Trial.trial_index()
        Trial.next()
        local i1 = Trial.trial_index()
        assert(i1 == i0 + 1, 'trial index did not advance')
    ",
    )
    .unwrap();
}

#[test]
fn trial_set_block() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run("Trial.set_block(2)").unwrap();
}

#[test]
fn data_record_writes_to_file() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        Data.record({
            condition = 'congruent',
            correct   = true,
            rt_ms     = 423.5,
        })
        Data.record({
            condition = 'incongruent',
            correct   = false,
            rt_ms     = 612.0,
        })
        Data.save()
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
    assert!(contents.contains("423"));
}

#[test]
fn data_record_increments_trial_index() {
    let dir = tempfile::tempdir().unwrap();
    let host = make_host(dir.path());
    host.run(
        "
        local i0 = Data.trial_index()
        Data.record({ x = 1 })
        local i1 = Data.trial_index()
        Data.record({ x = 2 })
        local i2 = Data.trial_index()
        assert(i1 == i0 + 1, 'index did not advance after first record')
        assert(i2 == i0 + 2, 'index did not advance after second record')
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
fn lua_table_to_json_object() {
    use crate::script::api_data::lua_table_to_json;
    let lua = mlua::Lua::new();
    let tbl: mlua::Table = lua
        .load(
            r#"
        return { condition = "congruent", correct = true, rt = 423.5 }
    "#,
        )
        .eval()
        .unwrap();

    let json = lua_table_to_json(&tbl).unwrap();
    assert_eq!(json["condition"], "congruent");
    assert_eq!(json["correct"], true);
    assert!((json["rt"].as_f64().unwrap() - 423.5).abs() < 0.001);
}

#[test]
fn lua_table_to_json_array() {
    use crate::script::api_data::lua_table_to_json;
    let lua = mlua::Lua::new();
    let tbl: mlua::Table = lua.load("return {10, 20, 30}").eval().unwrap();

    let json = lua_table_to_json(&tbl).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0], 10);
    assert_eq!(json[2], 30);
}

#[test]
fn lua_table_to_json_nested() {
    use crate::script::api_data::lua_table_to_json;
    let lua = mlua::Lua::new();
    let tbl: mlua::Table = lua
        .load(
            r#"
        return { outer = { inner = "value" } }
    "#,
        )
        .eval()
        .unwrap();

    let json = lua_table_to_json(&tbl).unwrap();
    assert_eq!(json["outer"]["inner"], "value");
}
