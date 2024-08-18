use std::{
    fmt::Write,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use crate::{instruction::Instruction, window::Window};

/// Total size of the available memory.
/// 4KB in total.
const RAM_SIZE: usize = 0x1000;

/// Start of the program.
/// The bytes beofre are traditionally taken up by the interpreter
const PROGRAM_START: usize = 0x200;

/// VF register.
///
/// This should not be used by programs, mainly used to store flags.
const REG_VF: usize = 15;

pub struct Interpreter {
    /// Memory of the interpreter.
    /// Should have a length of [`Interpreter::RAM_SIZE`].
    memory: Vec<u8>,
    /// Program Counter
    /// Points to the address that is currently being executed.
    program_counter: u16,
    /// General purpose registers V0 to VF.
    ///
    /// VF should not be accessed by the program.
    registers: [u8; 16],
    /// 16-bit register.
    /// Usually holds memory adresses.
    address_register: u16,
    /// Special sound register.
    ///
    /// If non-zero, it is automatically decremented at a rate of 60 Hz.
    /// When non-zero, a sound is played.
    sound_register: u8,
    /// Special timer register.
    ///
    /// If non-zero, it is automatically decremented at a rate of 60 Hz.
    timer_register: u8,
    /// Stack pointer
    ///
    /// Points to the topmost level of the stack.
    stack_pointer: u8,
    /// Stack
    ///
    /// Stores the adresses that should be returned to once finished with a subroutine.
    stack: [u16; 16],
    /// Fame Buffer of the current window.
    ///
    /// Since the interpreter only supports a max width of 64 pixel,
    /// `u64`s are (mis-)used as bitfields.
    frame_buffer: Arc<RwLock<[u64; Window::HEIGHT]>>,
    /// Window that is used to display sprites, etc.
    window: Window,
}

impl Interpreter {
    /// Create a new interpreter with the given rom file.
    pub fn new(rom_file: Vec<u8>) -> Self {
        // set up a shared frame buffer between window and interpreter
        let frame_buffer = Arc::new(RwLock::new([0; Window::HEIGHT]));

        let mut interpreter = Self {
            memory: vec![0u8; RAM_SIZE],
            registers: [0; 16],
            address_register: 0,
            sound_register: 0,
            timer_register: 0,
            stack_pointer: 0,
            program_counter: PROGRAM_START as u16,
            stack: [0; 16],
            window: Window::new(Arc::clone(&frame_buffer)),
            frame_buffer,
        };

        // write font bytes into interpreter meory
        for (idx, digit) in Window::DIGITS.iter().enumerate() {
            interpreter.write_bytes(idx * digit.len(), digit);
        }
        // write rom file into memory
        interpreter.write_bytes(PROGRAM_START, &rom_file);
        interpreter
    }

    /// Executes the current program in memory.
    pub fn execute(&mut self) -> Result<(), String> {
        self.window.spawn();

        // rate at which timer/sound are decreased. Repsondeds to 60Hz, ~16.67ms
        let timer_cycle: Duration = Duration::from_secs_f64(1.0 / 60.0);
        let mut timer_clock = Instant::now();

        loop {
            // fetch next instruction
            let instruction_bytes = self
                .read_u16(self.program_counter as usize)
                .unwrap_or_default();
            if instruction_bytes == 0 {
                // likely found last instruction
                return Ok(());
            }

            let instruction = Instruction::try_from(instruction_bytes)?;
            // cycle until a draw call is found, for which we need to update the screen
            let is_draw_call = matches!(instruction, Instruction::Drw(..));
            // step to next instruction
            self.program_counter += 2;
            // TODO: stepdown timer regs
            self.execute_instruction(instruction)?;

            // decrement timer registers
            if timer_clock.elapsed() >= timer_cycle {
                self.timer_register = self.timer_register.saturating_sub(1);
                self.sound_register = self.sound_register.saturating_sub(1);
                timer_clock = Instant::now();
            }

            if is_draw_call {
                self.window.queue_draw();
            }
        }
    }

    /// Dumps the current memory state to stderr.
    pub fn dump_memory(&self) {
        self.memory
            .chunks(16 * 2)
            .map(|bytes| {
                bytes.iter().fold(
                    String::with_capacity(bytes.len() * 2),
                    |mut output, byte| {
                        let _ = write!(output, "{byte:02X}");
                        output
                    },
                )
            })
            .for_each(|line| eprintln!("{}", line));
    }

    /// Writes the given bytes to the memory, starting at the given offset.
    fn write_bytes(&mut self, address: usize, bytes: &[u8]) {
        self.memory
            .splice(address..(address + bytes.len()), bytes.iter().copied());
    }

    fn read_byte(&self, address: usize) -> Option<&u8> {
        self.memory.get(address)
    }

    fn read_u16(&self, address: usize) -> Option<u16> {
        let bytes = &self.memory[address..=(address + 1)];
        Some(u16::from_be_bytes(bytes.try_into().ok()?))
    }

    fn read_bytes(&self, address: usize, len: usize) -> &[u8] {
        &self.memory[address..(address + len)]
    }

    fn push_subroutine(&mut self, address: u16) {
        // safe current program counter
        self.stack_pointer += 1;
        self.stack[self.stack_pointer as usize] = self.program_counter;
        // jump to subroutine
        self.program_counter = address;
    }

    fn pop_subroutine(&mut self) {
        // pop to last address
        self.program_counter = self.stack[self.stack_pointer as usize];
        self.stack_pointer -= 1;
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), String> {
        match instruction {
            Instruction::Sys(addr) => self.push_subroutine(addr),
            Instruction::Cls => self.window.clear(),
            Instruction::Ret => self.pop_subroutine(),
            Instruction::JpAddr(addr) => self.program_counter = addr,
            Instruction::Call(addr) => self.push_subroutine(addr),
            Instruction::SeVxByte(reg, byte) => {
                if self.registers[reg as usize] == byte {
                    self.program_counter += 2;
                }
            }
            Instruction::SneVxByte(reg, byte) => {
                if self.registers[reg as usize] != byte {
                    self.program_counter += 2;
                }
            }
            Instruction::SeVxVy(reg_x, reg_y) => {
                if self.registers[reg_x as usize] == self.registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::LdVxByte(reg, byte) => self.registers[reg as usize] = byte,
            Instruction::AddVxByte(reg, byte) => {
                self.registers[reg as usize] = self.registers[reg as usize].wrapping_add(byte)
            }
            Instruction::LdVxVy(reg_x, reg_y) => {
                self.registers[reg_x as usize] = self.registers[reg_y as usize]
            }
            Instruction::Or(reg_x, reg_y) => {
                self.registers[reg_x as usize] |= self.registers[reg_y as usize]
            }
            Instruction::And(reg_x, reg_y) => {
                self.registers[reg_x as usize] &= self.registers[reg_y as usize]
            }
            Instruction::Xor(reg_x, reg_y) => {
                self.registers[reg_x as usize] ^= self.registers[reg_y as usize]
            }
            Instruction::AddVxVy(reg_x, reg_y) => {
                let res = (self.registers[reg_x as usize] as u16)
                    + (self.registers[reg_y as usize] as u16);
                // truncate value to last 8 bits
                self.registers[reg_x as usize] = res as u8;
                self.registers[REG_VF] = (res > 255) as u8;
            }
            Instruction::Sub(reg_x, reg_y) => {
                let x = self.registers[reg_x as usize];
                let y = self.registers[reg_y as usize];
                self.registers[reg_x as usize] = x.wrapping_sub(y);
                self.registers[REG_VF] = (x >= y) as u8;
            }
            Instruction::Shr(reg_x, reg_y) => {
                let y = self.registers[reg_y as usize];
                self.registers[reg_x as usize] = y >> 1;
                self.registers[REG_VF] = y & 1;
            }
            Instruction::Subn(reg_x, reg_y) => {
                let x = self.registers[reg_x as usize];
                let y = self.registers[reg_y as usize];
                self.registers[reg_x as usize] = y.wrapping_sub(x);
                self.registers[REG_VF] = (y >= x) as u8;
            }
            Instruction::Shl(reg_x, reg_y) => {
                let y = self.registers[reg_y as usize];
                self.registers[reg_x as usize] = y << 1;
                self.registers[REG_VF] = (y >> 7) & 1;
            }
            Instruction::SneVxVy(reg_x, reg_y) => {
                if self.registers[reg_x as usize] != self.registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::LdIAddr(addr) => self.address_register = addr,
            Instruction::JpV0Addr(addr) => self.program_counter = addr + self.registers[0] as u16,
            Instruction::Rnd(reg, byte) => {
                let rand = rand::random::<u8>();
                self.registers[reg as usize] = rand & byte;
            }
            Instruction::Drw(reg_x, reg_y, n) => self.draw_sprite(
                self.registers[reg_x as usize],
                self.registers[reg_y as usize],
                n,
            ),
            Instruction::Skp(reg) => {
                if self.window.is_key_pressed(self.registers[reg as usize]) {
                    self.program_counter += 2;
                }
            }
            Instruction::Sknp(reg) => {
                if !self.window.is_key_pressed(self.registers[reg as usize]) {
                    self.program_counter += 2;
                }
            }
            Instruction::LdVxDt(reg) => self.registers[reg as usize] = self.timer_register,
            Instruction::LdVxK(reg) => {
                self.registers[reg as usize] = self.window.wait_for_key_press()
            }
            Instruction::LdDtVx(reg) => self.timer_register = self.registers[reg as usize],
            Instruction::LdStVx(reg) => self.sound_register = self.registers[reg as usize],
            Instruction::AddIVx(reg) => {
                self.address_register += self.registers[reg as usize] as u16
            }
            Instruction::LdFVx(reg) => {
                self.address_register = (self.registers[reg as usize].wrapping_mul(5)) as u16;
            }
            Instruction::LdBVx(reg) => {
                let mut val = self.registers[reg as usize];
                let mut i = 0;
                while val > 0 {
                    let n = val % 10;
                    val /= 10;
                    self.write_bytes(self.address_register as usize + i, &[n]);
                    i += 1;
                }
            }
            Instruction::LdIVx(reg) => {
                self.registers
                    .into_iter()
                    .take(reg as usize + 1)
                    .enumerate()
                    .for_each(|(i, reg)| {
                        self.write_bytes((self.address_register as usize) + i, &[reg])
                    });
            }
            Instruction::LdVxI(reg) => {
                for i in 0..=(reg as usize) {
                    self.registers[i] =
                        *self.read_byte(self.address_register as usize + i).unwrap();
                }
            }
        };
        Ok(())
    }

    /// Draw the sprite located at [`Self::address_register`]
    /// to [`Self::address_register`] + `n` starting at (`x`, `y`).
    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        let draw_bytes = self
            .read_bytes(self.address_register as usize, n as usize)
            .to_vec();
        let mut frame_buffer = self.frame_buffer.write().unwrap();
        for (i, byte) in draw_bytes.into_iter().enumerate() {
            let coord = (y as usize + i) % Window::HEIGHT;
            let original = frame_buffer[coord];
            // shift an addiontal 8 bits, so the byte is moved to the beginning
            let res = original ^ (byte as u64).rotate_right(x as u32 + 8);
            // check if any bits where erased (set to 0)
            self.registers[REG_VF] = ((original & !res) != 0) as u8;
            frame_buffer[coord] = res;
        }
    }
}
