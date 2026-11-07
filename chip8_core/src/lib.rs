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
        Self {
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
        }
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -=1;
        self.stack[self.stack_pointer as  usize]
    }
}

