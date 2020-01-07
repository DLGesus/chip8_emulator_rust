use chip8_emulator::Chip8_CPU;
use std::sync::Mutex;
use std::sync::Arc;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;


fn main() {
    let mut chip8 = Chip8_CPU::new();
    chip8.init();
    chip8.load("Pong.ch8");

    loop {
        chip8.cycle();

        if chip8.draw_flag {
            //println!("----------------------------------------");
            chip8.draw_graphics();
        }

        chip8.setKeys();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

}
