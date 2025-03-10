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

    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],    // 1-bit screen or B&W

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
            
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT], 
            
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

        let op: u16 = higher_byte << 8 | lower_byte;     // Combine the bytes into a 16-bit value

        self.pc += 2;   // increment the program counter by 2 since opcodes are 16-bits and memory are only 8-bits

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

            // CLS (00e0): CLEAR SCREEN
            (0, 0, 0xE, 0) => {
                self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // RET (00ee): RETURN from Subroutine
            (0, 0, 0xE, 0xE) => {
                let return_add: u16 = self.pop();
                self.pc = return_add;
            }

            // JMP NNN (1nnn): JUMP to address nnn
            (0, _, _, _) => {
                self.pc = nnn as u16;
            }

            // CALL NNN (2nnn): CALL Subroutine, similar to JMP but needs to remember where it came from
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
                    print!("OVERFLOW: opcode 7xnn, x={}, nn={}", x, nn);
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
                let mut rng = rand::thread_rng();
                let random: u8 = rng.r#gen();                   // r#gen instead of gen since rust has a keyword gen https://doc.rust-lang.org/edition-guide/rust-2024/gen-keyword.html

                self.v_reg[x] = random & nnn as u8;
            }

            // DRW Vx, Vy, nibble (Dxyn): DRAW n-byte sprite starting at I (Vx, Vy), set VF = collision
            // This opcode executes by reading n-bytes starting at the address I. These bytes contains the sprite that would drawn on screen
            // at the starting coordinates (Vx, Vy). 
            // Every row of the sprite is saved in an address I, if the sprite is 3px tall, this means the rows are in addresses: I, I+1, I+2
            // These addresses are 1 byte or 8-bits wide, where every bit of the address represents the width of the columns
            // This pretty much guarantees that every sprite could be n pixels high and 8 pixels wide.
            // The sprites or the pixels are drawn using XOR operation
            // If this XOR causes the pixel to flip, we will set the Vf = 1
            (0xD, _, _, _) => {
                // x = x coordinate, y = y coordinate, n = sprite height

                self.v_reg[0xF] = 0;    // Reset every call to avoid issues if Vf is set in previous calls
                
                // We iterate per byte
                for row in 0..n {
                    
                    let addr: u16 = self.index_reg + row as u16;        // Get the address of sprite rows (I, I+1, I+2, ...)
                    let pixel_data: u8 = self.memory[addr as usize];    // Then find it in the RAM

                    let y: usize = (self.v_reg[y] as usize + row) % SCREEN_HEIGHT;  // Find the y coodinate of the sprite, use modulo to wrap around the screen
                    
                    // Now we iterate per bit from MSB to LSB
                    for column in 0..8{
                        
                        let x: usize = (self.v_reg[x] as usize + column) % SCREEN_WIDTH;    // Find the x coordinate of the sprite, use modulo to wrap around the screen

                        let sprite_pixel: u8 = (pixel_data >> (7 - column as u8)) & 1;      // Extract each bit and check then flip if value is 1, // We can honestly use if else here, but using AND operation is just the same

                        let screen_idx: usize = x + (SCREEN_WIDTH * y);                     // Flip the Vf flag if there is a collision (or if the sprite pixel erased the current pixel)
                        self.v_reg[0xF] |= sprite_pixel & self.screen[screen_idx];
                        
                        self.screen[screen_idx] ^= sprite_pixel                             // Flip the pixel on the screen
                    }
                }
            }

            // SKP Vx (Ex9E): SKIP NEXT instruction if value in Vx is pressed
            (0xE, _, 9, 0xE) => {
                let key: u8 = self.v_reg[x];

                if self.keypad[key as usize]{
                    self.pc += 2;
                }
            }

            // SKNP Vx (ExA1): SKIP NEXT instruction if value in Vx is not pressed
            (0xE, _, 0xA, 1) => {
                let key: u8 = self.v_reg[x];

                if !self.keypad[key as usize]{
                    self.pc += 2;
                }
            }

            // LD Vx, DT (Fx07): SET Vx to the value of the delay timer
            (0xF, _, 0, 7) => {
                self.v_reg[x] = self.delay_timer;
            }

            // LD Vx, K (Fx08): Wait for key press and SET Vx to the value of the key pressed
            (0xF, _, 0, 8) => {
                let mut pressed: bool = false;

                for keys in 0..KEYPAD_SIZE {
                    if self.keypad[keys] {
                        self.v_reg[x] = keys as u8;
                        pressed = true;
                        break;  
                    }
                }
                
                if !pressed{
                    self.pc -= 2    // If no key is pressed, jump back to the previous instruction which is this same instruction
                }
            }

            // LD DT, Vx (Fx15): SET delay timer from Vx
            (0xF, _, 1, 5) => {
                self.delay_timer = self.v_reg[x];
            }

            // LD ST, Vx (Fx18): SET sound timer from Vx
            (0xF, _, 1, 8) => {
                self.sound_timer = self.v_reg[x];
            }

            // ADD I, Vx (Fx1E): ADD I = I + VX
            (0xF, _, 1, 0xE) => {
                let (sum, overflow) = self.index_reg.overflowing_add(self.v_reg[x] as u16);

                if overflow {
                    print!("OVERFLOW: opcode Fx1E, x={}", x);
                }

                self.index_reg = sum;
            }

            // LD F, Vx (Fx29): SET I = location of sprite for Vx
            (0xF, _, 2, 9) => {
                let font_digit = self.v_reg[x] as u16;

                self.index_reg = font_digit * 5;    // multiply by 5 since fonts are 5 bytes long/tall, we can use this to find the starting address of the font
            }

            // LD B, Vx (Fx33): SET BCD representation of Vx in memory locations I, I+1, I+2
            (0xF, _, 3, 3) => {
                let dec = self.v_reg[x] as f32;

                self.memory[self.index_reg as usize] = (dec / 100.0).floor() as u8;
                self.memory[self.index_reg as usize + 1] = ((dec / 10.0) % 10.0).floor() as u8;
                self.memory[self.index_reg as usize + 2] = (dec % 10.0) as u8;
            }

            // LD [I], Vx (Fx55): STORE V0 to Vx in memory starting from address I
            (0xF, _, 5, 5) => {
                let start_idx = self.index_reg as usize;
                for index in 0..=x{
                    self.memory[index + start_idx] = self.v_reg[index];
                }
            }

            // LD Vx, [I] (Fx65): SET/LOAD V0 to Vx from memory starting from address I
            (0xF, _, 6, 6) => {
                let start_idx = self.v_reg[0] as usize;

                for index in 0..=x{
                    self.v_reg[index] = self.memory[index + start_idx];
                }
            }

            // Unknown
            (_, _, _, _) => {
                println!("Unknown opcode: 0x{:04X}", op);
            }
        }
    }
}

