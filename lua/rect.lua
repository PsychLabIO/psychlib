local instr = Stim.text("Follow the squares with your cursor", {
    size = 0.05,
    align = "center",
    y = 0.2
})

Trial.show(instr, 1500)

for i = 1, 5 do
    local x = Rand.float(-0.8, 0.8)
    local y = Rand.float(-0.8, 0.8)

    local dot = Stim.rect(x, y, 0.05, 0.05, "white")

    Trial.show(dot, 300)
    Trial.blank(200)
end
