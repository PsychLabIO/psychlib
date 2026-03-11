local function make_node(run_fn)
    return { run = run_fn }
end

local _screen_h = (type(_psychlib_screen_h) == "number" and _psychlib_screen_h) or 768

local TEXT_SIZE_BODY     = math.floor(_screen_h * 0.040)
local TEXT_SIZE_FEEDBACK = math.floor(_screen_h * 0.052)
local FIX_ARM_LEN        = math.floor(_screen_h * 0.028)
local FIX_THICKNESS      = math.max(2, math.floor(_screen_h * 0.004))

function Timeline()
    local self = {
        _nodes = {},
    }

    function self:add(node)
        table.insert(self._nodes, node)
    end

    --- Declare the output format before experiment:run().
    function self:set_format(format)
        _psychlib_set_format(format)
    end

    function self:run()
        ctx.trial_index = 0
        ctx.block       = 0
        ctx.trial       = nil
        ctx.last_response = nil

        for _, node in ipairs(self._nodes) do
            node:run()
        end
    end

    return self
end

--- Run a list of nodes in order.
function Sequence(nodes)
    return make_node(function()
        for _, node in ipairs(nodes) do
            node:run()
        end
    end)
end

function ForBlocks(n, fn)
    return make_node(function()
        for block = 1, n do
            ctx.block = block
            local node = fn(block)
            node:run()
        end
    end)
end

function ForTrials(list, fn)
    return make_node(function()
        for _, trial in ipairs(list) do
            ctx.trial_index = ctx.trial_index + 1
            ctx.trial       = trial
            ctx.last_response = nil
            local node = fn(trial)
            node:run()
        end
    end)
end

--- Run `node` if `predicate()` returns true; optionally run `else_node`.
function If(predicate, node, else_node)
    return make_node(function()
        if predicate() then
            node:run()
        elseif else_node ~= nil then
            else_node:run()
        end
    end)
end

--- Run `node` repeatedly while `predicate()` returns true.
function Loop(predicate, node)
    return make_node(function()
        while predicate() do
            node:run()
        end
    end)
end

--- Show a text screen and wait for a keypress. If `duration` is set (ms),
--- auto-advance after that time instead.
function Instructions(opts)
    assert(type(opts.text) == "string", "Instructions: text is required")
    return make_node(function()
        local stim = Stim.text(opts.text, {
            size  = opts.size  or TEXT_SIZE_BODY,
            color = opts.color or "white",
            align = opts.align or "center",
            font  = opts.font,
        })
        if opts.duration then
            _psychlib_show(stim, opts.duration)
        else
            _psychlib_show(stim, nil)
            _psychlib_wait_key({})
        end
        _psychlib_blank(500)
    end)
end

--- Show a fixation cross for a fixed duration (ms).
function Fixation(opts)
    assert(type(opts.duration) == "number", "Fixation: duration is required")
    return make_node(function()
        local stim = Stim.fixation({
            color     = opts.color     or "white",
            arm_len   = opts.arm_len   or FIX_ARM_LEN,
            thickness = opts.thickness or FIX_THICKNESS,
        })
        _psychlib_show(stim, opts.duration)
    end)
end

--- Show `stim`, collect a keypress, write result to ctx.last_response.
function Stimulus(opts)
    assert(opts.stim    ~= nil,             "Stimulus: stim is required")
    assert(type(opts.keys) == "table",      "Stimulus: keys must be a table")
    assert(type(opts.timeout) == "number",  "Stimulus: timeout is required")
    return make_node(function()
        _psychlib_show(opts.stim, nil)
        local resp = _psychlib_wait_key({ keys = opts.keys, timeout = opts.timeout })

        local correct = nil
        if opts.correct_key ~= nil then
            correct = resp ~= nil and resp.key == opts.correct_key
        end

        ctx.last_response = {
            key       = resp and resp.key or nil,
            rt_ms     = resp and resp.rt_ms or nil,
            timed_out = resp == nil,
            correct   = correct,
        }
    end)
end

--- Silent blank screen for a fixed duration (ms).
function Blank(opts)
    assert(type(opts.duration) == "number", "Blank: duration is required")
    return make_node(function()
        _psychlib_blank(opts.duration)
    end)
end

--- Show feedback text based on ctx.last_response.correct.
function Feedback(opts)
    assert(type(opts.correct_text)   == "string", "Feedback: correct_text is required")
    assert(type(opts.incorrect_text) == "string", "Feedback: incorrect_text is required")
    assert(type(opts.duration)       == "number", "Feedback: duration is required")
    return make_node(function()
        assert(ctx.last_response ~= nil,
            "Feedback: no last_response in ctx, place Feedback after a Stimulus node")
        local is_correct = ctx.last_response.correct
        local text  = is_correct and opts.correct_text   or opts.incorrect_text
        local color = is_correct
            and (opts.correct_color   or opts.color or "white")
            or  (opts.incorrect_color or opts.color or "white")
        local stim = Stim.text(text, {
            size  = opts.size  or TEXT_SIZE_FEEDBACK,
            color = color,
            align = opts.align or "center",
            font  = opts.font,
        })
        _psychlib_show(stim, opts.duration)
    end)
end

function EndScreen(opts)
    assert(type(opts.text) == "string", "EndScreen: text is required")
    return make_node(function()
        local stim = Stim.text(opts.text, {
            size  = opts.size  or TEXT_SIZE_BODY,
            color = opts.color or "white",
            align = opts.align or "center",
            font  = opts.font,
        })
        if opts.duration then
            _psychlib_show(stim, opts.duration)
        else
            _psychlib_show(stim, nil)
            _psychlib_wait_key({})
        end
    end)
end

--- Write a trial row.
function Record(fields)
    assert(type(fields) == "table", "Record: fields must be a table")
    return make_node(function()
        local row = {}

        row.trial_index = ctx.trial_index
        row.block       = ctx.block

        if ctx.last_response ~= nil then
            local r = ctx.last_response
            row.response_key = r.key
            row.rt_ms        = r.rt_ms
            row.timed_out    = r.timed_out
            if r.correct ~= nil then
                row.correct = r.correct
            end
        end

        for k, v in pairs(fields) do
            row[k] = v
        end

        _psychlib_write_trial(row)
    end)
end

--- Flush and close the data file.
function Save()
    return make_node(function()
        _psychlib_save()
    end)
end

--- Return a shallow-shuffled copy of `list` using the host RNG.
function Shuffle(list)
    local copy = {}
    for i, v in ipairs(list) do copy[i] = v end
    for i = #copy, 2, -1 do
        local j = Rand.int(1, i)
        copy[i], copy[j] = copy[j], copy[i]
    end
    return copy
end