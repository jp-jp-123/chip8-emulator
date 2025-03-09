use std::collections::VecDeque;
use rand::Rng;

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
    memory: [u8; RAM_SIZE],         // memory, 4kB/4096 bytes large
    v_reg: [u8; V_REG_SIZE],        // V-Register, 8 bits
    index_reg: u16,                 // index register, 12 bytes
    stack: [u16; STACK_REG_SIZE],   // stack, 16 bytes (we could probs convert this vecdeque instead TODO)
    stack_pointer: u16,             // stack pointer, may or may not be used later if std library isn't compatible to WebAssembly
    
    sound_timer: u8,                // sound timer, 8 bits
    delay_timer: u8,                // delay timer, 8 bits

    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],    // 1-bit screen or B&W

    keypad: [bool; KEYPAD_SIZE]     // keypad, 16 keys (0 - 9, A - F)

}

impl Chip8 {
    pub fn new() -> Self{
        let mut ram: [u8; 4096] = [0u8; RAM_SIZE];              // initialize memory with 0s
        ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);     // copy FONTSET to the ram

        Self {
            pc: START_ADDRESS, 
            memory: ram, 
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

    fn pop(&mut self) -> u16{
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    // vecDeque for stack, just in case they actually work in WebAssembly
    fn stdPush(&mut self, val: u16){
        let mut mem_stack: VecDeque<u16> = VecDeque::from(self.stack);
        mem_stack.push_back(val); 

        self.stack.copy_from_slice(&mem_stack.make_contiguous()[..STACK_REG_SIZE]);
    }

    fn stdPop(&mut self){
        let mut mem_stack: VecDeque<u16> = VecDeque::from(self.stack);
        mem_stack.pop_back(); 

        self.stack.copy_from_slice(&mem_stack.make_contiguous()[..STACK_REG_SIZE]);
    }

    fn movePC(&mut self){
        self.pc += 2;
    }

    pub fn tick(&mut self){
        // FETCH
        let op: u16 = self.fetch();

        // DECODE & EXECUTE
        self.execute(op);
        
    }

    pub fn fetch(&mut self) -> u16{
        // chip8 stores opcodes in Big-Endian, hence higher bytes are stored in lower memory
        let higher_byte: u16 = self.memory[self.pc as usize] as u16;         // Fetch the higher byte
        let lower_byte: u16 = self.memory[(self.pc + 1) as usize] as u16;    // Fetch the lower byte

        let op: u16 = higher_byte << 8 | lower_byte;     // combine the bytes into a 16-bit value

        self.movePC();

        return op;
    }

    pub fn timers(&mut self){
        if self.delay_timer > 0{
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0{
            if self.sound_timer == 1{
                // PLAY BEEP
                // TODO
            }
            self.sound_timer -= 1;
        }
    }

    pub fn execute(&mut self, op: u16){
        // DECODE
        let nibbles: (u8, u8, u8, u8) = (
            ((op & 0xF000) >> 12) as u8,    // extract the 1st nibble
            ((op & 0x0F00) >> 8) as u8,     // extract the 2nd nibble
            ((op & 0x00F0) >> 4) as u8,     // extract the 3rd nibble
            (op & 0x000F) as u8           // extract the 4th nibble
        );

        let x: usize = nibbles.1 as usize;
        let y: usize = nibbles.2 as usize;
        let n: usize = nibbles.3 as usize;
        let nn: u8 = (op & 0x00FF) as u8;
        let nnn: usize = (op & 0x0FFF) as usize;

        match(nibbles.0, nibbles.1, nibbles.2, nibbles.3){
            // NOP (0000): DO NOTHING
            (0, 0, 0, 0) => {
                return;
            }

            // CLS (00E0): CLEAR SCREEN
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // RET (00EE): RETURN FROM SUBROUTINE
            (0, 0, 0xE, 0xE) => {
                let return_add: u16 = self.pop();
                self.pc = return_add;
            }

            // JMP NNN (1NNN): JUMP TO ADDRESS
            (0, _, _, _) => {
                self.pc = nnn as u16;
            }

            // CALL NNN (2NNN): CALL SUBROUTINE, SIMILAR TO JMP BUT NEEDS TO REMEMBER WHERE IT CAME FROM
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = nnn as u16
            }

            // SE Vx, byte (3xnn): SKIP IF VX == NN
            (3, _, _, _) => {
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // SNE VX, byte (4xnn): SKIP IF VX != NN
            (4, _, _, _) => {
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }   
            
            // SE Vx, Vy (5xy0): SKIP IF VX == VY
            (5, _, _, 0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // LD Vx, byte (6xnn): LOAD Vx = nn
            (6, _, _, _) => {
                self.v_reg[x] = nn
            }

            // ADD Vx, byte (7xnn): ADD Vx = Vx + nn
            (7, _, _, _) => {

                // self.v_reg[x] += nn;                                                 // panic on overflow
                // self.v_reg[x] = self.v_reg[x].wrapping_add(nn);                      // silent wrap around on overflow
                let (sum, overflow) = self.v_reg[x].overflowing_add(nn);      // wrap around on overflow and notifies if it happens

                if overflow {
                    print!("OVERFLOW")
                }

                self.v_reg[x] = sum
                
            }

            // LD Vx, Vy (8xy0): LOAD Vx = Vy
            (8, _, _, 0) => {
                self.v_reg[x] = self.v_reg[y]
            }

            // OR Vx, Vy (8xy1): Vx = Vx | Vy
            (8, _, _, 1) => {
                self.v_reg[x] |= self.v_reg[y];
            }

            // AND Vx, Vy (8xy2): Vx = Vx & Vy
            (8, _, _, 2) => {
                self.v_reg[x] &= self.v_reg[y];
            }

            // XOR Vx, Vy (8xy3): Vx = Vx ^ Vy
            (8, _, _, 3) => {
                self.v_reg[x] ^= self.v_reg[y];
            }

            // ADD Vx, Vy (8xy4): Vx = Vx + Vy. Set VF for carry
            (8, _, _, 4) => {
                let (sum, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                self.v_reg[0xF] = if carry {1} else {0};

                self.v_reg[x] = sum
            }

            // SUB Vx, Vy (8xy5): Vx = Vx - Vy, SET VF for borrow
            (8, _, _, 5) => {
                let (diff, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                self.v_reg[0xF] = if borrow {0} else {1};   // Note that SET VF = NOT borrow

                self.v_reg[x] = diff
            }

            // Vx SHR 1 (8xy6): SET VF for Vx's least significant bit, then SET Vx = Vx >> 1 (basically Vx / 2), 
            (8, _, _, 6) => {
                self.v_reg[0xF] = self.v_reg[x] & 0x1;

                self.v_reg[x] >>= 1
            }

            // SUBN Vx, Vy (8xy7): Vx = Vy - Vx, SET VF for borrow
            (8, _, _, 7) => {
                let (diff, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                self.v_reg[0xF] = if borrow {0} else {1};   // Note that SET VF = NOT borrow

                self.v_reg[x] = diff
            }

            // Vx SHL 1 (8xyE): SET VF = Vx's most significant bit, then SET Vx = Vx << 1 (basically Vx * 2)
            (8, _, _, 0xE) => {
                self.v_reg[0xF] = (self.v_reg[x] & 0x80) >> 7;

                self.v_reg[x] <<= 1
            }
            
            // SNE Vx, Vy (9xy0): SKIP NEXT instruction Vx != Vy
            (9, _, _, 0) => {
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // LD I, addr (Annn): LOAD I (index register) = nnn
            (0xA, _, _, _) => {
                self.index_reg = nnn as u16
            }

            // JP V0, addr (Bnnn): JUMP to addr + V0
            (0xB, _, _, _) => {
                self.pc = self.v_reg[0] as u16 + nnn as u16;
            }

            // RND Vx, byte (Cxnn): SET Vx = random byte AND nnn
            (0xC, _, _, _) => {
                let mut rng:u8 = rand::thread_rng();
                self.v_reg[x] = rng & nnn as u8;
            }

            // Unknown
            (_, _, _, _) => {
                println!("Unknown opcode: 0x{:04X}", op);
            }
        }
    }
}

