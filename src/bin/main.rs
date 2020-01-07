use chip8_emulator::Chip8_CPU;

fn main() {
    let mut chip8 = Chip8_CPU::new();
    chip8.init();
    chip8.load("Pong.ch8");

    loop {
        chip8.cycle();

        if chip8.draw_flag {
            println!("----------------------------------------");
            chip8.draw_graphics();
        }

        //chip8.setKeys

    }

}
