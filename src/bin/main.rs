use chip8_emulator::Chip8CPU;

fn main() {
    let mut chip8 = Chip8CPU::new();
    chip8.init();
    chip8.load("Pong.ch8");

    loop {
        chip8.cycle();

        if chip8.draw_flag {
            //println!("----------------------------------------");
            chip8.draw_graphics();
        }

        chip8.set_keys();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

}
