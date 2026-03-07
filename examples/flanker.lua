-- Eriksen Flanker Task

local N_BLOCKS = 2
local TRIALS_PER_CELL = 5
local FIXATION_MS = 500
local RESPONSE_MS = 1500
local ITI_MS = 800
local RESPONSE_KEYS = { "left", "right" }

local function make_arrows(direction, congruent)
    local centre  = direction == "left" and "<" or ">"
    local flanker
    if congruent then
        flanker = direction == "left" and "<<" or ">>"
    else
        flanker = direction == "left" and ">>" or "<<"
    end
    return Stim.text(flanker .. centre .. flanker,
        { size = 0.08, color = "white", align = "center" })
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

local experiment = Timeline()
experiment:set_format("json")

experiment:add(Instructions({
    text = "Flanker Task\n\n" ..
           "Press LEFT for <   Press RIGHT for >\n" ..
           "Respond to the CENTRE arrow only.\n\n" ..
           "Press any key to begin.",
}))

experiment:add(ForBlocks(N_BLOCKS, function(block)
    return Sequence({
        ForTrials(Shuffle(make_trials()), function(trial)
            local correct_key = trial.direction == "left" and "left" or "right"
            return Sequence({
                Fixation({ duration = FIXATION_MS }),
                Stimulus({
                    stim = make_arrows(trial.direction, trial.congruent),
                    keys = RESPONSE_KEYS,
                    timeout = RESPONSE_MS,
                    correct_key = correct_key,
                }),
                Blank({ duration = ITI_MS }),
                Record({
                    direction = trial.direction,
                    congruent = trial.congruent,
                }),
            })
        end),

        If(function() return ctx.block < N_BLOCKS end,
            Instructions({
                text = "Block " .. block .. " of " .. N_BLOCKS .. " complete.\n\n" ..
                       "Take a short break.\n" ..
                       "Press any key when ready.",
            })
        ),
    })
end))

experiment:add(EndScreen({
    text = "Task complete. Thank you!",
    duration = 2000,
}))

experiment:add(Save())

experiment:run()