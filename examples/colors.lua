local colors = {
    "red",
    "green",
    "blue",
    "#ffaa00",
    Stim.rgb(255, 0, 255)
}

for i, c in ipairs(colors) do
    local x = -0.8 + (i - 1) * 0.4
    local rect = Stim.rect(x, 0.0, 0.15, 0.15, c)
    Trial.show(rect, 500)
end

Trial.blank(1000)
