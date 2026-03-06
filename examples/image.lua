Trial.preload_image("stimuli/cat.jpg")


local instr = Stim.text("Images work!", {
    size = 0.05,
    align = "center",
    y = 0.2
})

local img = Stim.image("stimuli/cat.jpg", { hw = 0.4, hh = 0.6 })

Trial.show(instr, 1500)
Trial.show(img, 2000)