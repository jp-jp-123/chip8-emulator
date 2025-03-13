use chip8_engine::*;
use wasm-bindgen::prelude::*;

#[wasm_bindgen]
pub struct Chip8EngineWasm {
    chip8: Chip8;
}

#[wasm_bindgen]
impl Chip8EngineWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
         chip8 = Chip8::new(true);
    }

    #[wasm_bindgen]
    pub fn tick(&mut self){
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn timers(&mut self){
        self.chip8.timers();
    }
}

