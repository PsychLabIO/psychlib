-- Eriksen Flanker Task

local N_BLOCKS = 2
local TRIALS_PER_CELL = 5
local FIXATION_MS = 500
local STIMULUS_MS = 500
local RESPONSE_MS = 1500
local ITI_MS = 800

local RESPONSE_KEYS = { "left", "right" }

local fix = Stim.fixation({ color = "white", arm_len = 0.02, thickness = 0.004 })

local function make_arrows(direction, congruent)
    local centre = direction == "left" and "<" or ">"
    local flanker
    if congruent then
        flanker = direction == "left" and "<<" or ">>"
    else
        flanker = direction == "left" and ">>" or "<<"
    end
    local display = flanker .. centre .. flanker
    return Stim.text(display, { size = 0.08, color = "white", align = "center" })
end

local function make_trials()
    local list = {}
    for _, dir in ipairs({ "left", "right" }) do
        for _, cong in ipairs({ true, false }) do
            for _ = 1, TRIALS_PER_CELL do
                list[#list + 1] = { direction = dir, congruent = cong }
            end
        end
    end
    return list
end

local function shuffle(t)
    for i = #t, 2, -1 do
        local j = Rand.int(1, i)
        t[i], t[j] = t[j], t[i]
    end
    return t
end

local function show_instructions()
    local msg = Stim.text(
        "Flanker Task\n\n" ..
        "Press LEFT for < Press RIGHT for >\n" ..
        "Respond to the CENTRE arrow only.\n\n" ..
        "Press any key to begin.",
        { size = 0.04, color = "white", align = "center" }
    )
    Trial.show(msg)
    Trial.wait_key()
    Trial.blank(500)
end

local function show_break(block, n_blocks)
    if block >= n_blocks then return end
    local msg = Stim.text(
        "Block " .. block .. " of " .. n_blocks .. " complete.\n\n" ..
        "Take a short break.\n" ..
        "Press any key when ready.",
        { size = 0.04, color = "white", align = "center" }
    )
    Trial.show(msg)
    Trial.wait_key()
    Trial.blank(500)
end

local function run_trial(trial)
    local stim = make_arrows(trial.direction, trial.congruent)

    Trial.show(fix, FIXATION_MS)

    local onset = Trial.show(stim)

    local resp = Trial.wait_key({
        keys    = RESPONSE_KEYS,
        timeout = RESPONSE_MS,
        onset   = onset,
    })

    Trial.blank()

    Trial.blank(ITI_MS)

    local correct_key = trial.direction == "left" and "left" or "right"
    local correct = resp ~= nil and resp.key == correct_key

    return {
        correct = correct,
        rt_ms = resp and resp.rt_ms or nil,
        responded = resp ~= nil,
        key = resp and resp.key or nil,
    }
end

show_instructions()

for block = 1, N_BLOCKS do
    Trial.set_block(block)
    local trials = shuffle(make_trials())

    for _, trial in ipairs(trials) do
        local result = run_trial(trial)

        Data.record({
            block = block,
            trial = Trial.trial_index(),
            direction = trial.direction,
            congruent = trial.congruent,
            correct = result.correct,
            rt_ms = result.rt_ms,
            responded = result.responded,
            key = result.key,
        })

        Trial.next()
    end

    show_break(block, N_BLOCKS)
end

Trial.show(Stim.text(
    "Task complete. Thank you!",
    { size = 0.05, color = "white", align = "center" }
), 2000)

Data.save()
