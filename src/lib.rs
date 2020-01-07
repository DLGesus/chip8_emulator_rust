extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::Sdl;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use rand::prelude::*;
use std::sync::Mutex;
use std::sync::Arc;

static FONTS: [u8; 80] =
[ 
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8_CPU {
    // 4k emulated memory
    // 0x000-0x1FF = Chip 8 Interpreter
    // 0x050-0x0A0 = 4x5 pixel font set
    // 0x200-0xFFF = Program ROM and Work RAM
    memory: [u8; 4096],
    // 15 8-bit Registers V0-VE for general use, and VF(8-bit) as a flag for some instructions
    v: [u8; 16],
    // Index register for memory
    i: u16,
    // Program Counter for instructions
    pc: u16,
    // Storage of the graphics array, 64 pixels by 32 pixels
    gfx: [u8; 64*32],
    // Delay Timer, counts down to zero in 60 Hz intervals
    dt: u8,
    // Sound Timer, counts down to zero in 60 Hz intervals. System buzzer sounds at zero.
    st: u8,
    // 16 levels of stack for subroutines
    stack: Vec<u16>,
    // Current state of the key pressed for the HEX based keypad
    pub keys: [bool; 16],
    event_pump: sdl2::EventPump,
    // Current opcode
    opcode: u16,
    pub draw_flag: bool,
    canvas: sdl2::render::WindowCanvas,
}

impl Chip8_CPU {
    pub fn new() -> Chip8_CPU{
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Chip_8 Emulator", 64*5, 32*5).position_centered().build().unwrap();
        Chip8_CPU {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0,
            gfx: [0; 64*32],
            dt: 0,
            st: 0,
            stack: Vec::with_capacity(16),
            keys: [false; 16],
            event_pump: sdl_context.event_pump().unwrap(),
            opcode: 0,
            draw_flag: true,
            canvas: window.into_canvas().build().unwrap(),
        }
    }

    pub fn init(&mut self) {
        self.pc = 0x200;
        self.opcode = 0;
        self.i = 0;
        self.stack = Vec::with_capacity(16);
        self.keys = [false; 16];
        self.draw_flag = true;
        self.canvas.clear();
        self.canvas.present();

        for i in 0..80 {
            self.memory[i] = FONTS[i];
        }
    }

    pub fn load(&mut self, filename: &str) {
        let mut file = File::open(filename).unwrap();
    
        let n = file.read(&mut self.memory[0x200..]).unwrap();
        //println!("{}", n);
        let mut i = 0x200;
        while i < n+0x200 {
            let opcode: u16 = (self.memory[i] as u16) << 8 | self.memory[i+1] as u16;
            //print!("{:#x?}, ", opcode);
            i += 2;
        }
    }

    pub fn cycle(&mut self) {
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        //println!("{:#x?}", opcode);

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0xE0 => self.i_clr_00e0(opcode),
                    0xEE => self.i_ret_00ee(opcode),
                    0x0 | _ => self.i_sys_0nnn(opcode),
                }
            },
            0x1000 => self.i_jp_1nnn(opcode),
            0x2000 => self.i_call_2nnn(opcode),
            0x3000 => self.i_se_3xkk(opcode),
            0x4000 => self.i_sne_4xkk(opcode),
            0x5000 => self.i_se_5xy0(opcode),
            0x6000 => self.i_ld_6xkk(opcode),
            0x7000 => self.i_add_7xkk(opcode),
            0x8000 => {
                match opcode & 0x000F {
                    0x0 => self.i_ld_8xy0(opcode),
                    0x1 => self.i_or_8xy1(opcode),
                    0x2 => self.i_and_8xy2(opcode),
                    0x3 => self.i_xor_8xy3(opcode),
                    0x4 => self.i_add_8xy4(opcode),
                    0x5 => self.i_sub_8xy5(opcode),
                    0x6 => self.i_shr_8xy6(opcode),
                    0x7 => self.i_subn_8xy7(opcode),
                    0xE => self.i_shl_8xye(opcode),
                    _ => {},
                }
            },
            0x9000 => self.i_sne_9xy0(opcode),
            0xA000 => self.i_ld_annn(opcode),
            0xB000 => self.i_jp_bnnn(opcode),
            0xC000 => self.i_rnd_cxkk(opcode),
            0xD000 => self.i_drw_dxyn(opcode),
            0xE000 => {
                match opcode & 0x00FF {
                    0x9E => self.i_skp_ex9e(opcode),
                    0xA1 => self.i_sknp_exa1(opcode),
                    _ => {},
                }
            },
            0xF000 => {
                match opcode & 0x00FF {
                    0x07 => self.i_ld_fx07(opcode),
                    0x0A => self.i_ld_fx0a(opcode),
                    0x15 => self.i_ld_fx15(opcode),
                    0x18 => self.i_ld_fx18(opcode),
                    0x1E => self.i_add_fx1e(opcode),
                    0x29 => self.i_ld_fx29(opcode),
                    0x33 => self.i_ld_fx33(opcode),
                    0x55 => self.i_ld_fx55(opcode),
                    0x65 => self.i_ld_fx65(opcode),
                    _ => {},
                }
            },
            _ => {},
        }

        if self.dt > 0 {
            self.dt -= 1;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        if self.st > 0 {
            if self.st == 1 {
                println!("IM MISTER MEESEEKS LOOK AT ME!");
            }
            self.st -= 1;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    pub fn setKeys(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Num0) => self.keys[0x0] = true,
                        Some(Keycode::Num1) => self.keys[0x1] = true,
                        Some(Keycode::Num2) => self.keys[0x2] = true,
                        Some(Keycode::Num3) => self.keys[0x3] = true,
                        Some(Keycode::Num4) => self.keys[0x4] = true,
                        Some(Keycode::Num5) => self.keys[0x5] = true,
                        Some(Keycode::Num6) => self.keys[0x6] = true,
                        Some(Keycode::Num7) => self.keys[0x7] = true,
                        Some(Keycode::Num8) => self.keys[0x8] = true,
                        Some(Keycode::Num9) => self.keys[0x9] = true,
                        Some(Keycode::A) => self.keys[0xa] = true,
                        Some(Keycode::B) => self.keys[0xb] = true,
                        Some(Keycode::C) => self.keys[0xc] = true,
                        Some(Keycode::D) => self.keys[0xd] = true,
                        Some(Keycode::E) => self.keys[0xe] = true,
                        Some(Keycode::F) => self.keys[0xf] = true,
                        _ => {},
                    }
                },
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Num0) => self.keys[0x0] = false,
                        Some(Keycode::Num1) => self.keys[0x1] = false,
                        Some(Keycode::Num2) => self.keys[0x2] = false,
                        Some(Keycode::Num3) => self.keys[0x3] = false,
                        Some(Keycode::Num4) => self.keys[0x4] = false,
                        Some(Keycode::Num5) => self.keys[0x5] = false,
                        Some(Keycode::Num6) => self.keys[0x6] = false,
                        Some(Keycode::Num7) => self.keys[0x7] = false,
                        Some(Keycode::Num8) => self.keys[0x8] = false,
                        Some(Keycode::Num9) => self.keys[0x9] = false,
                        Some(Keycode::A) => self.keys[0xa] = false,
                        Some(Keycode::B) => self.keys[0xb] = false,
                        Some(Keycode::C) => self.keys[0xc] = false,
                        Some(Keycode::D) => self.keys[0xd] = false,
                        Some(Keycode::E) => self.keys[0xe] = false,
                        Some(Keycode::F) => self.keys[0xf] = false,
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    }

    pub fn draw_graphics(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(0, 255, 0));
        for j in 0..32 {
            for i in 0..64 {
                let pixel_location = (i + (j * 64)) as usize;
                let pixel = self.gfx[pixel_location];
                if pixel == 1 {
                    for x in 0..5 {
                        for y in 0..5 {
                            self.canvas.draw_point(sdl2::rect::Point::new(x + (5*i), y + (5*j))).unwrap();
                        }
                    }
                }
            }
        }
        self.canvas.present();
        self.draw_flag = false;
    }

    fn i_sys_0nnn(&mut self, opcode: u16) {
        self.pc += 2;
        return;
    }
    
    fn i_clr_00e0(&mut self, opcode: u16) {
        self.gfx = [0; 64*32];
        self.draw_flag = true;
        self.pc += 2;
    }

    fn i_ret_00ee(&mut self, opcode: u16) {
        self.pc = self.stack.pop().unwrap();
        self.pc += 2;
    }

    fn i_jp_1nnn(&mut self, opcode: u16) {
        let addr: u16 = opcode & 0x0FFF;
        self.pc = addr;
    }

    fn i_call_2nnn(&mut self, opcode: u16) {
        let addr: u16 = opcode & 0x0FFF;
        self.stack.push(self.pc);
        self.pc = addr;
    }

    fn i_se_3xkk(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        if self.v[x as usize] == kk {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_sne_4xkk(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        if self.v[x as usize] != kk {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_se_5xy0(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        if self.v[x as usize] == self.v[y as usize] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_ld_6xkk(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        self.v[x as usize] = kk;
        self.pc += 2;
    }

    fn i_add_7xkk(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
        self.pc += 2;
    }

    fn i_ld_8xy0(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[x as usize] = self.v[y as usize];
        self.pc += 2;
    }

    fn i_or_8xy1(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[x as usize] |= self.v[y as usize];
        self.pc += 2;
    }

    fn i_and_8xy2(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[x as usize] &= self.v[y as usize];
        self.pc += 2;
    }

    fn i_xor_8xy3(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[x as usize] ^= self.v[y as usize];
        self.pc += 2;
    }

    fn i_add_8xy4(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        let result: u16 = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        self.v[15] = match result > 255 {
            true => 1,
            false => 0,
        };
        self.v[x as usize] = (result & 0xFF) as u8;
        self.pc += 2;
    }

    fn i_sub_8xy5(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[15] = match self.v[x as usize] > self.v[y as usize] {
            true => 1,
            false => 0,
        };
        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
        self.pc += 2;
    }

    fn i_shr_8xy6(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.v[15] = self.v[x as usize] & 0x01;
        self.v[x as usize] /= 2;
        self.pc += 2;
    }

    fn i_subn_8xy7(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        self.v[15] = match self.v[y as usize] > self.v[x as usize] {
            true => 1,
            false => 0,
        };
        self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
        self.pc += 2;
    }

    fn i_shl_8xye(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.v[15] = match self.v[x as usize] & 0x80 {
            0x1 => 1,
            _ => 0,
        };
        self.v[x as usize] *= 2;
        self.pc += 2;
    }

    fn i_sne_9xy0(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        if self.v[x as usize] != self.v[y as usize] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_ld_annn(&mut self, opcode: u16) {
        let addr: u16 = opcode & 0x0FFF;
        self.i = addr;
        self.pc += 2;
    }

    fn i_jp_bnnn(&mut self, opcode: u16) {
        let addr: u16 = opcode & 0x0FFF;
        self.pc = addr + self.v[0] as u16;
    }

    fn i_rnd_cxkk(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        let rand_num: u8 = rand::random::<u8>();
        self.v[x as usize] = rand_num & kk;
        self.pc += 2;
    }

    fn i_drw_dxyn(&mut self, opcode: u16) {
        let x: u16 = self.v[((opcode & 0x0F00) >> 8) as usize] as u16;
        let y: u16 = self.v[((opcode & 0x00F0) >> 4) as usize] as u16;
        let n: u16 = opcode & 0x000F;

        //println!("DRAWING AT x: {}, y: {}", x, y);

        self.v[15] = 0;
        for j in 0..n {
            let pixel: u8 = self.memory[(j as u16 + self.i) as usize];
            for i in 0..8 {
                if pixel & (0x80 >> i) != 0 {
                    let pixel_location = ((x + i) % 64 + (((y + j) % 32) * 64)) as usize;
                    if self.gfx[pixel_location] == 1 {
                        self.v[15] = 1;
                    }
                    self.gfx[pixel_location] ^= 1;
                }
            }
        }

        self.draw_flag = true;
        self.pc += 2;
    }

    fn i_skp_ex9e(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let vx: u8 = self.v[x as usize];

        if self.keys[vx as usize] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_sknp_exa1(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let vx: u8 = self.v[x as usize];

        if !self.keys[vx as usize] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn i_ld_fx07(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.v[x as usize] = self.dt;
        self.pc += 2;
    }

    fn i_ld_fx0a(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;

        let mut key_pressed: usize = 16;

        while key_pressed > 15 {
            for (index, key) in self.keys.iter().enumerate() {
                if *key {
                    key_pressed = index;
                }
            }
        }

        self.v[x as usize] = key_pressed as u8;
        self.pc += 2;
    }

    fn i_ld_fx15(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.dt = self.v[x as usize];
        self.pc += 2;
    }

    fn i_ld_fx18(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.st = self.v[x as usize];
        self.pc += 2;
    }

    fn i_add_fx1e(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.i += self.v[x as usize] as u16;
        self.pc += 2;
    }

    fn i_ld_fx29(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        self.i = 5 * self.v[x as usize] as u16;
        self.pc += 2;
    }

    fn i_ld_fx33(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        let vx: u8 = self.v[x as usize];
        self.memory[self.i as usize] = vx / 100;
        self.memory[1 + self.i as usize] = (vx / 10) % 10;
        self.memory[2 + self.i as usize] = vx % 10;
        self.pc += 2;
    }

    fn i_ld_fx55(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        for i in 0..=x {
            self.memory[(i + self.i) as usize] = self.v[i as usize];
        }
        self.pc += 2;
    }

    fn i_ld_fx65(&mut self, opcode: u16) {
        let x: u16 = (opcode & 0x0F00) >> 8;
        for i in 0..=x {
            self.v[i as usize] = self.memory[(i + self.i) as usize];
        }
        self.pc += 2;
    }
     
}