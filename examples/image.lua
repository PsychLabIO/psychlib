Trial.preload_image("stimuli/funkydude.jpg")


local instr = Stim.text("Images work!", {
    size = 0.05,
    align = "center",
    y = 0.2
})

local img = Stim.image("stimuli/funkydude.jpg", { hw = 0.4, hh = 0.4 })

Trial.show(instr, 1500)
Trial.show(img, 2000)