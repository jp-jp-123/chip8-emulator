import init, * as wasm from "./wasm.js"

const WIDTH = 64
const HEIGHT = 32
const SCALE = 15
const TICKS_PER_FRAME = 10
let anim_frame = 0

const canvas = document.getElementById("canvas")
canvas.width = WIDTH * SCALE
canvas.height = HEIGHT * SCALE

const ctx = canvas.getContext("2d")
ctx.fillStyle = "black"
ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE)

const input = document.getElementById("fileinput")
async function run() {
    await init()

    let chip8 = new wasm.Chip8EngineWasm()

    document.addEventListener("keydown", function(evt){
        chip8.keypress(evt, true)
    })

    document.addEventListener("keyup", function(evt){
        chip8.keypress(evt, false)
    })

    // Load game
    input.addEventListener("change", function(evt){
        // Stop previous game from rendering if it exists
        if (anim_frame != 0){
            window.cancelAnimationFrame(anim_frame)
        }

        // Get file
        let file = evt.target.files[0]
        if(!file){
            alert("Failed to read file")
            return
        }

        // Load game as Uint8, send to wasm, and start main loop
        let rom_file = new FileReader()
        rom_file.onload = function(e){
            let buffer = rom_file.result
            const rom = new Uint8Array(buffer)
            chip8.reset()
            chip8.load_rom(rom)
            gameloop(chip8)
        }
        rom_file.readAsArrayBuffer(file)
    }, false)
}

function gameloop(chip8){
    for (let i = 0; i < TICKS_PER_FRAME; i++){
        chip8.tick()
    }
    chip8.timers()

    ctx.fillStyle = "black"
    ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE)

    ctx.fillStyle = "white"
    chip8.draw_screen(SCALE)

    anim_frame = window.requestAnimationFrame(() => {
        gameloop(chip8)
    })
}

run().catch(console.error)
