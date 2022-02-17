use rand::prelude::ThreadRng;
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Chip8 {
    ram: [u8; 0x1000],
    vx: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    display: [[bool; 64]; 32],
    rng: ThreadRng,
    input: [bool; 16],
}

const LETTER_SPRITES: [u8; 0x50] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];


impl Chip8 {
    pub fn new(path: &Path) -> Self {
        let mut ram: [u8; 0x1000] = [0; 0x1000];
        ram[..0x50].copy_from_slice(&LETTER_SPRITES);
        let mut file = File::open(path).unwrap();
        let _ = file.read(&mut ram[0x200..]).unwrap();

        let vx: [u8; 16] = [0; 16];

        Self {
            ram,
            vx,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            display: [[false; 64]; 32],
            rng: rand::thread_rng(),
            input: [false; 16],
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn run_inst(&mut self) -> bool {
        //let msb = self.ram[self.pc as usize];
        //let lsb = self.ram[self.pc as usize + 1];
        let instr = u16::from_be_bytes(
            self.ram[self.pc as usize..=self.pc as usize + 1]
                .try_into()
                .unwrap(),
        );
        let mut should_render = false;

        let nnn = instr & 0x0fff;

        let opcode: u8 = (instr >> 12) as u8;
        let ix: u8 = (instr >> 8 & 0xf) as u8;
        let iy: u8 = (instr >> 4 & 0xf) as u8;
        let nibble: u8 = (instr & 0xf) as u8;

        let kk = (instr & 0xff) as u8;

        match opcode {
            0 => {
                match instr {
                    0x00E0 => {
                        should_render = true;
                        self.display = [[false; 64]; 32];
                    } // CLS
                    0x00EE => {
                        self.pc = self.pop() // RET
                    }
                    _ => panic!("invalid opcode"),
                }
            }
            1 => {
                self.pc = nnn - 2; // JP addr
            }
            2 => {
                self.push(self.pc); // CALL addr
                self.pc = nnn - 2;
            }
            3 => {
                if self.vx[ix as usize] == kk {
                    // SE Vx, byte
                    self.pc += 2;
                }
            }
            4 => {
                if self.vx[ix as usize] != kk {
                    // SNE Vx, byte
                    self.pc += 2;
                }
            }
            5 => {
                assert_eq!(nibble, 0);
                if self.vx[ix as usize] == self.vx[iy as usize] {
                    //  SE Vx, Vy
                    self.pc += 2;
                }
            }
            6 => {
                self.vx[ix as usize] = kk; // LD Vx, byte
            }
            7 => {
                self.vx[ix as usize] = self.vx[ix as usize].overflowing_add(kk).0;
                // ADD Vx, byte
            }
            8 => {
                match nibble {
                    0 => self.vx[ix as usize] = self.vx[iy as usize], // LD Vx, Vy
                    1 => self.vx[ix as usize] |= self.vx[iy as usize], // OR Vx, Vy
                    2 => self.vx[ix as usize] &= self.vx[iy as usize], // AND Vx, Vy
                    3 => self.vx[ix as usize] ^= self.vx[iy as usize], // XOR Vx, Vy
                    4 => {
                        let (res, carry) =
                            self.vx[ix as usize].overflowing_add(self.vx[iy as usize]); // ADD Vx, Vy
                        self.vx[ix as usize] = res;
                        self.vx[0xf] = carry as u8;
                    }
                    5 => {
                        let (res, carry) =
                            self.vx[ix as usize].overflowing_sub(self.vx[iy as usize]); // SUB Vx, Vy
                        self.vx[ix as usize] = res;
                        self.vx[0xf] = !carry as u8;
                    }
                    6 => {
                        self.vx[0xf] = self.vx[ix as usize] & 1; // SHR Vx {, Vy}
                        self.vx[ix as usize] >>= 1;
                    }
                    7 => {
                        let (res, carry) =
                            self.vx[iy as usize].overflowing_sub(self.vx[ix as usize]); // SUBN Vx, Vy
                        self.vx[ix as usize] = res;
                        self.vx[0xf] = !carry as u8;
                    }
                    0xe => {
                        self.vx[0xf] = self.vx[ix as usize] >> 7; // SHL Vx {, Vy}
                        self.vx[ix as usize] <<= 1;
                    }
                    _ => panic!("invalid opcode"),
                }
            }
            9 => {
                assert_eq!(nibble, 0);
                if self.vx[ix as usize] != self.vx[iy as usize] {
                    // SNE Vx, Vy
                    self.pc += 2
                }
            }
            0xa => self.i = nnn,                          // LD I, addr
            0xb => self.pc = self.vx[0] as u16 + nnn - 2, // JP V0, addr
            0xc => self.vx[ix as usize] = self.rng.gen::<u8>() & kk, // RND Vx, byte
            0xd => {
                let mut overlap: u8 = 0;
                // DRW Vx, Vy, nibble
                let bx = self.vx[ix as usize];
                let by = self.vx[iy as usize];
                let n = nibble;
                for y in 0..n {
                    let byt = self.ram[self.i as usize + y as usize];
                    for x in 0..8 {
                        if (byt >> (7 - x)) & 1 == 1 {
                            let dy = (by as u16 + y as u16) % 32;
                            let dx = (bx as u16 + x as u16) % 64;
                            if self.display[dy as usize][dx as usize] {
                                overlap = 1;
                            }
                            self.display[dy as usize][dx as usize] =
                                !self.display[dy as usize][dx as usize];
                        }
                    }
                    should_render = true;
                }
                self.vx[0xf] = overlap;
            }
            0xe => {
                match kk {
                    0x9e => {
                        if self.input[self.vx[ix as usize] as usize] {
                            self.pc += 2
                        }
                    } // SKP Vx
                    0xa1 => {
                        if !self.input[self.vx[ix as usize] as usize] {
                            self.pc += 2
                        }
                    } // SKNP Vx
                    _ => panic!("invalid opcode"),
                }
            }
            0xf => {
                match iy {
                    0 => {
                        match nibble {
                            7 => self.vx[ix as usize] = self.delay_timer, //  LD Vx, DT
                            0xa => {
                                self.vx[ix as usize] = 0;
                                for (i, pressed) in self.input.iter().enumerate() {
                                    if *pressed {
                                        self.vx[ix as usize] = i as u8;
                                        break;
                                    }
                                }
                                if self.vx[ix as usize] == 0 {
                                    self.pc -= 2;
                                    should_render = true;
                                }
                                
                            }
                            _ => panic!("Invalid opcode"),
                        }
                    }
                    1 => {
                        match nibble {
                            5 => self.delay_timer = self.vx[ix as usize], // LD DT, Vx
                            8 => self.sound_timer = self.vx[ix as usize], // LD ST, Vx
                            0xe => self.i = self.i.overflowing_add(self.vx[ix as usize] as u16).0, // ADD I, Vx
                            _ => panic!("invalid opcode"),
                        }
                    }
                    2 => {
                        assert_eq!(nibble, 9);
                        self.i = 0x5 * (self.vx[ix as usize] as u16);
                    }
                    3 => {
                        assert_eq!(nibble, 3);
                        let num = self.vx[ix as usize];
                        let hundreds = num / 100;
                        let tens = (num - hundreds * 100) / 10;
                        let ones = num - hundreds * 100 - tens * 10;
                        self.ram[self.i as usize] = hundreds;
                        self.ram[self.i as usize + 1] = tens;
                        self.ram[self.i as usize + 2] = ones;
                    }
                    5 => {
                        assert_eq!(nibble, 5);
                        for x in 0..=ix {
                            self.ram[(self.i + x as u16) as usize] = self.vx[x as usize];
                        }
                    }
                    6 => {
                        assert_eq!(nibble, 5);
                        for x in 0..=ix {
                            self.vx[x as usize] = self.ram[(self.i + x as u16) as usize];
                        }
                    }
                    _ => panic!("invalid opcode"),
                }
            }

            _ => panic!("invalid opcode"),
        };

        self.pc += 2;

        should_render
    }

    pub fn run_frame(&mut self, input: [bool; 16]) -> &[[bool; 64]; 32] {
        self.input = input;

        for _ in 0..10 {
            if self.run_inst() {
                break;
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        &self.display
    }

    //    fn updateInput(&mut self, ...)
}
