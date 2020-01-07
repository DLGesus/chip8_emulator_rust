use chip8_emulator::Chip8CPU;

fn main() {
    let mut chip8 = Chip8CPU::new();
    chip8.init();
    chip8.load("games\\Tetris [Fran Dachille, 1991].ch8");

    loop {
        chip8.cycle();

        if chip8.draw_flag {
            //println!("----------------------------------------");
            chip8.draw_graphics();
        }

        match chip8.set_keys() {
            true => break,
            false => {},
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

}
