use chip8_engine::*;
use std::env;

use sdl2::event::Event;

const SCALE: u32 = 15;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;

fn main() {
    // Command Line argument
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // Initialize SDL2 Window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
                .window("CHIP-8 EMULATOR", WINDOW_WIDTH, WINDOW_HEIGHT)
                .position_centered()
                .opengl()
                .build()
                .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();
}