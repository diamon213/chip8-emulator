//TODO import display and font set
use crate::keypad::Keypad;
//TODO import ComplementaryMultiplyWithCarryGen

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    // index register
    pub i: u16,
    // program counter
    pub pc: u16,
    // registers labeled V0-VF -> 0-F -> 0-15
    pub vx: [u8; 16],
    // 4k memory, 4096 (0x2000)
    pub mem: [u8; 4096],
    // stack
    pub ret_stack: Vec<u16>,
    // stack pointer
    pub sp: u8,
    // delay timer
    pub dt: u8,
    // keypad
    pub keypad: Keypad,

    // display
    //TODO pub display: Display,

    // random number generator
    //TODO pub rand: ComplementaryMultiplyWithCarryGen
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            i: 0,
            pc: 0,
            vx: [0; 16],
            mem: [0; 4096],
            ret_stack: Vec::<u16>::new(),
            sp: 0,
            dt: 0,
            keypad: Keypad::new(),
            //TODO display
            //TODO rand
        }
    }

    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = PROGRAM_START;
        self.vx = [0; 16];
        self.mem = [0; 4096];
        self.ret_stack = Vec::<u16>::new();
        self.sp = 0;
        self.dt = 0;

        //TODO self.rng = rand::thread_rng();
        //TODO self.display.cls();
        //TODO fill memory with font set
    }

    pub fn execute_cycle(&mut self) {
        let opcode: u16 = read_word(self.mem, self.pc);
        self.process_opcode(opcode);
    }

    pub fn decrement_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    pub fn read_word(memory: [u8; 4096], index: u16) -> u16{
        (memory[index as usize] as u16) << 8
            | (memory[(index + 1) as usize] as u16)
    }

    fn process_opcode(&mut self, opcode: u16) {

        // extract various opcode attributes
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        // opcode gets split
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        // increment the program counter
        self.pc += 2;

        println!("{}, {}, {}, {}", op_1, op_2, op_3, op_4);


        match (op_1, op_2, op_3, op_4) {
            // CLS
            (0, 0, 0xE, 0) => self.display.cls(),
            // RET
            (0, 0, 0xE, 0xE) => {
                self.sp = self.sp - 1;
                self.pc = self.stack[self.sp as usize];
            },
            // JP
            (0x1, _, _, _) => self.pc = nnn,
            // CALL
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp + 1;
                self.pc = nnn;
            },
            // SE Vx KK
            (0x3, _, _, _) => self.pc += if vx == kk { 2 } else { 0 },
            // SNE Vx KK
            (0x4, _, _, _) => self.pc += if vx != kk { 2 } else { 0 },
            // SE Vx Vy
            (0x5, _, _, _) => self.pc += if vx == vy { 2 } else { 0 },
            // LD Vx
            (0x6, _, _, _) => self.v[x] = kk,
            // ADD Vx, byte
            (0x7, _, _, _) => self.v[x] += kk,
            // LD Vx, Vy
            (0x8, _, _, 0x0) => self.v[x] = self.v[y],
            // OR Vx, Vy
            (0x8, _, _, 0x1) => self.v[x] = self.v[x] | self.v[y],
            // AND Vx, Vy
            (0x8, _, _, 0x2) => self.v[x] = self.v[x] & self.v[y],
            // XOR Vx, Vy
            (0x8, _, _, 0x3) => self.v[x] = self.v[x] ^ self.v[y],
            // ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                match overflow {
                    true => self.v[0xF] = 1,
                    false => self.v[0xF] = 0,
                }
                self.v[x] = res;
            }
            // SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
                match overflow {
                    true => self.v[0xF] = 0,
                    false => self.v[0xF] = 1,
                }
                self.v[x] = res;
            }
            // SHR Vx
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            // SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
                match overflow {
                    true => self.v[0xF] = 0,
                    false => self.v[0xF] = 1,
                }
                self.v[x] = res;
            },
            // SHL Vx
            (0x8, _, _, 0xE) => {
                self.v[0xF] = self.v[x] & 0x80;
                self.v[x] <<= 1;
            }
            // SNE Vx Vy
            (0x9, _, _, _) => self.pc += if vx != vy { 2 } else { 0 },
            // LD I
            (0xA, _, _, _) => self.i = nnn,
            // JP V0
            (0xB, _, _, _) => self.pc = nnn + self.v[0] as u16,
            // RND
            (0xC, _, _, _) => self.v[x] = self.rand.random() as u8 & kk,
            // DRW
            (0xD, _, _, _) => {
                let collision = self.display.draw(vx as usize, vy as usize,
                                                  &self.memory[self.i as usize..(self.i + n as u16) as usize]);
                self.v[0xF] = if collision { 1 } else { 0 };
            }
            // SKP Vx
            (0xE, _, 0x9, 0xE) => self.pc += if self.keypad.is_key_down(vx) { 2 } else { 0 },
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => self.pc += if self.keypad.is_key_down(vx) { 0 } else { 2 },
            // LD Vx, DT
            (0xF, _, 0x0, 0x7) => self.v[x] = self.dt,
            // LD Vx, K
            (0xF, _, 0x0, 0xA) => {
                self.pc -= 2;
                for (i, key) in self.keypad.keys.iter().enumerate() {
                    if *key == true {
                        self.v[x] = i as u8;
                        self.pc += 2;
                    }
                }
            },
            // LD DT, Vx
            (0xF, _, 0x1, 0x5) => self.dt = self.v[x],
            // ADD I, Vx
            (0xF, _, 0x1, 0xE) => self.i = self.i + self.v[x] as u16,
            // LD F, Vx
            (0xF, _, 0x2, 0x9) => self.i = vx as u16 * 5,
            // LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = (vx % 100) % 10;
            },
            // LD [I], Vx
            (0xF, _, 0x5, 0x5) => self.memory[(self.i as usize)..(self.i + x as u16 + 1) as usize]
                .copy_from_slice(&self.v[0..(x as usize + 1)]),
            // LD Vx, [I]
            (0xF, _, 0x6, 0x5) => self.v[0..(x as usize + 1)]
                .copy_from_slice(&self.memory[(self.i as usize)..(self.i + x as u16 + 1) as usize]),
            (_, _, _, _) => ()
        }
    }
}



