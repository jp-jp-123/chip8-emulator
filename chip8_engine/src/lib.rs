use std::collections::VecDeque;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const V_REG_SIZE: usize = 16;
const STACK_REG_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;

const START_ADDRESS: u16 = 0x200;

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

pub struct Chip8{
    pc: u16,                        // program counter, 12 bytes
    memory: [u16; RAM_SIZE],        // memory, 4kB/4096 bytes large
    v_reg: [u8; V_REG_SIZE],        // V-Register, 8 bits
    index_reg: u16,                 // index register, 12 bytes
    stack: [u16; STACK_REG_SIZE],    // stack, 16 bytes
    stack_pointer: u16,             // stack pointer, may or may not be used later if std library isn't compatible to WebAssembly
    
    sound_timer: u8,                // sound timer, 8 bits
    delay_timer: u8,                // delay timer, 8 bits

    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],    // 1-bit screen or B&W

    keypad: [bool; KEYPAD_SIZE]     // keypad, 16 keys (0 - 9, A - F)

}

impl Chip8 {
    pub fn new() -> Self{
        Self {
            pc: START_ADDRESS, 
            memory: [0; RAM_SIZE], 
            v_reg: [0; V_REG_SIZE], 
            index_reg: 0, 
            stack: [0; STACK_REG_SIZE],
            stack_pointer: 0,
            
            sound_timer: 0,
            delay_timer: 0, 
            
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT], 
            
            keypad: [false; KEYPAD_SIZE]
        }
    }

    fn push(&mut self, val: u16){
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    fn pop(&mut self){
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize];
    }

    fn stdPush(&mut self, val: u16){
        let mut mem_stack: VecDeque<u16> = VecDeque::from(self.stack);
        mem_stack.push_back(val); 
    }

    fn stdPop(&mut self){
        let mut mem_stack: VecDeque<u16> = VecDeque::from(self.stack);
        mem_stack.pop_back(); 
    }
}

