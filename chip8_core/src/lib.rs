use core::borrow;
use rand::random;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEY_COUNT: usize = 16;

const START_ADDRESS: u16 = 0x200;

pub struct Emulator{
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; REGISTER_COUNT],
    i_register: u16,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    keys: [bool; KEY_COUNT],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator{
    pub fn new() -> Self {
        let mut new_emulator = Self {
            program_counter: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; REGISTER_COUNT],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; KEY_COUNT],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; REGISTER_COUNT];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; KEY_COUNT];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -=1;
        self.stack[self.stack_pointer as  usize]
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        
        self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let opcode = (higher_byte << 8) | lower_byte;
        self.program_counter += 2;
        opcode
    }

    pub fn tick_timers(&mut self){
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //sound
            }
            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = (opcode & 0x000F);

        match (digit1, digit2, digit3, digit4) {
            // NOP  nop
            (0, 0, 0, 0) => return,
            // CLS  clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET  return from subroutine
            (0, 0, 0xE, 0xE) => {
                let return_addr = self.pop();
                self.program_counter = return_addr;
            },
            // JMP NNN  jump
            (1, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = nnn;
            },
            // CALL NNN  call subroutine
            (2, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            },
            // SKIP VX == NN  skip line if VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.program_counter += 2;
                }
            },
            // SKIP VX != NN  skip line if VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.program_counter += 2;
                }
            },
            // SKIP VX == VY  skip line if VX == VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] == self.v_registers[y] {
                    self.program_counter += 2;
                }
            },
            // VX = NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_registers[x] = nn;
            },
            // VX += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            },
            // VX = VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            },
            // bitwise operations
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] |= self.v_registers[y];
            },
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] &= self.v_registers[y];
            },
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] ^= self.v_registers[y];
            },
            // VX += VY
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX -= VY
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX >>= 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = lsb;
            },
            // VX = VY - VX
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX <<= 1
            (8, _, _, 8) => {
                let x = digit2 as usize;
                let msb = (self.v_registers[x] >> 7) & 1;
                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = msb;
            },
            // SKIP VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            },
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.i_register = nnn;
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = (self.v_registers[0] as u16) + nnn;
            },
            // VX = random() & NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                let rng: u8 = random();
                self.v_registers[x] = rng & nn;
            }
            (_, _, _, _,) => unimplemented!(),
        }
    }


}

