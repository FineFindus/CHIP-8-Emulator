#[derive(Debug)]
pub enum Instruction {
    /// Jump to a machine code routine at `addr`.
    ///
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    Sys(u16),
    /// Clear the display.
    Cls,
    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    Ret,
    /// Jump to location `addr`.
    ///
    /// The interpreter sets the program counter to `addr`.
    JpAddr(u16),
    /// Call subroutine at `addr`.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to `addr`.
    Call(u16),
    /// Skip next instruction if `Vx` = `kk`.
    ///
    /// The interpreter compares register `Vx` to `kk`, and if they are equal, increments the program counter by 2.
    SeVxByte(u8, u8),
    /// Skip next instruction if `Vx` != `kk`.
    ///
    /// The interpreter compares register `Vx` to `kk`, and if they are not equal, increments the program counter by 2.
    SneVxByte(u8, u8),
    /// Skip next instruction if `Vx` = `Vy`.
    ///
    /// The interpreter compares register `Vx` to register `Vy`, and if they are equal, increments the program counter by 2.
    SeVxVy(u8, u8),
    /// Set `Vx` = `kk`.
    ///
    /// The interpreter puts the value `kk` into register `Vx`.
    LdVxByte(u8, u8),
    /// Set `Vx` = `Vx` + `kk`.
    ///
    /// Adds the value `kk` to the value of register Vx, then stores the result in `Vx`.
    AddVxByte(u8, u8),
    ///Set `Vx` = Vy.
    ///
    ///Stores the value of register `Vy` in register `Vx`.
    LdVxVy(u8, u8),
    /// Set `Vx` = `Vx` OR Vy.
    ///
    /// Performs a bitwise OR on the values of `Vx` and Vy, then stores the result in `Vx`. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    Or(u8, u8),
    /// Set `Vx` = `Vx` AND Vy.
    ///
    /// Performs a bitwise AND on the values of `Vx` and Vy, then stores the result in `Vx`. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    And(u8, u8),
    /// Set `Vx` = `Vx` XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of `Vx` and Vy, then stores the result in `Vx`. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    Xor(u8, u8),
    /// Set `Vx` = `Vx` + Vy, set VF = carry.
    ///
    /// The values of `Vx` and `Vy` are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in `Vx`.
    AddVxVy(u8, u8),
    /// Set `Vx` = `Vx` - Vy, set VF = NOT borrow.
    ///
    /// If `Vx` > Vy, then VF is set to 1, otherwise 0. Then `Vy` is subtracted from Vx, and the results stored in `Vx`.
    Sub(u8, u8),
    //TODO: incorrect?
    /// Set `Vx` = `Vx` SHR 1.
    ///
    /// If the least-significant bit of `Vx` is 1, then VF is set to 1, otherwise 0. Then `Vx` is divided by 2.
    Shr(u8, u8),
    /// Set `Vx` = `Vy` - Vx, set VF = NOT borrow.
    ///
    /// If `Vy` > Vx, then VF is set to 1, otherwise 0. Then `Vx` is subtracted from Vy, and the results stored in `Vx`.
    Subn(u8, u8),
    //TODO: incorrect?
    /// Set `Vx` = `Vx` SHL 1.
    ///
    /// If the most-significant bit of `Vx` is 1, then VF is set to 1, otherwise to 0. Then `Vx` is multiplied by 2.
    Shl(u8, u8),
    /// Skip next instruction if `Vx` != Vy.
    ///
    /// The values of `Vx` and `Vy` are compared, and if they are not equal, the program counter is increased by 2.
    SneVxVy(u8, u8),
    /// Set I = `addr`.
    ///
    /// The value of register I is set to `addr`.
    LdIAddr(u16),
    /// Jump to location `addr` + `V0`.
    ///
    /// The program counter is set to `addr` plus the value of `V0`.
    JpV0Addr(u16),
    /// Set `Vx` = random byte AND `kk`.
    ///
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value `kk`. The results are stored in `Vx`. See instruction 8xy2 for more information on AND.
    Rnd(u8, u8),
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    Drw(u8, u8, u8),
    /// Skip next instruction if key with the value of `Vx` is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of `Vx` is currently in the down position, PC is increased by 2.
    Skp(u8),
    /// Skip next instruction if key with the value of `Vx` is not pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of `Vx` is currently in the up position, PC is increased by 2.
    Sknp(u8),
    /// Set `Vx` = delay timer value.
    ///
    /// The value of DT is placed into `Vx`.
    LdVxDt(u8),
    /// Wait for a key press, store the value of the key in `Vx`.
    ///
    /// All execution stops until a key is pressed, then the value of that key is stored in `Vx`.
    LdVxK(u8),
    /// Set delay timer = `Vx`.
    ///
    /// DT is set equal to the value of `Vx`.
    LdDtVx(u8),
    /// Set sound timer = `Vx`.
    ///
    /// ST is set equal to the value of `Vx`.
    LdStVx(u8),
    /// Set I = I + `Vx`.
    ///
    /// The values of I and `Vx` are added, and the results are stored in I.
    AddIVx(u8),
    /// Set I = location of sprite for digit `Vx`.
    ///
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of `Vx`. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    LdFVx(u8),
    /// Store BCD representation of `Vx` in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    LdBVx(u8),
    //TODO: incorrect?
    /// Store registers `V0` through `Vx` in memory starting at location I.
    ///
    /// The interpreter copies the values of registers `V0` through `Vx` into memory, starting at the address in I.
    LdIVx(u8),
    //TODO: incorrect?
    /// Read registers `V0` through `Vx` from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I into registers `V0` through `Vx`.
    LdVxI(u8),
}

impl TryFrom<u16> for Instruction {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let ident = (
            ((value >> 12) & 0xF),
            ((value >> 8) & 0xF),
            ((value >> 4) & 0xF),
            (value & 0xF),
        );

        let address = |x, y, k| (x << 8) | (y << 4) | k;
        let byte = |y, k| ((y << 4) | k) as u8;
        Ok(match ident {
            (0x0, 0x0, 0xE, 0x0) => Self::Cls,
            (0x0, 0x0, 0xE, 0xE) => Self::Ret,
            (0x0, x, y, k) => Self::Sys(address(x, y, k)),
            (0x1, x, y, k) => Self::JpAddr(address(x, y, k)),
            (0x2, x, y, k) => Self::Call(address(x, y, k)),
            (0x3, x, y, k) => Self::SeVxByte(x as u8, byte(y, k)),
            (0x4, x, y, k) => Self::SneVxByte(x as u8, byte(y, k)),
            (0x5, x, y, 0) => Self::SeVxVy(x as u8, y as u8),
            (0x6, x, y, k) => Self::LdVxByte(x as u8, byte(y, k)),
            (0x7, x, y, k) => Self::AddVxByte(x as u8, byte(y, k)),
            (0x8, x, y, 0x0) => Self::LdVxVy(x as u8, y as u8),
            (0x8, x, y, 0x1) => Self::Or(x as u8, y as u8),
            (0x8, x, y, 0x2) => Self::And(x as u8, y as u8),
            (0x8, x, y, 0x3) => Self::Xor(x as u8, y as u8),
            (0x8, x, y, 0x4) => Self::AddVxVy(x as u8, y as u8),
            (0x8, x, y, 0x5) => Self::Sub(x as u8, y as u8),
            (0x8, x, y, 0x6) => Self::Shr(x as u8, y as u8),
            (0x8, x, y, 0x7) => Self::Subn(x as u8, y as u8),
            (0x8, x, y, 0xE) => Self::Shl(x as u8, y as u8),
            (0x9, x, y, 0x0) => Self::SneVxVy(x as u8, y as u8),
            (0xA, x, y, k) => Self::LdIAddr(address(x, y, k)),
            (0xB, x, y, k) => Self::JpV0Addr(address(x, y, k)),
            (0xC, x, y, k) => Self::Rnd(x as u8, byte(y, k)),
            (0xD, x, y, k) => Self::Drw(x as u8, y as u8, k as u8),
            (0xE, x, 0x9, 0xE) => Self::Skp(x as u8),
            (0xE, x, 0xA, 0x1) => Self::Sknp(x as u8),
            (0xF, x, 0x0, 0x7) => Self::LdVxDt(x as u8),
            (0xF, x, 0x0, 0xA) => Self::LdVxK(x as u8),
            (0xF, x, 0x1, 0x5) => Self::LdDtVx(x as u8),
            (0xF, x, 0x1, 0x8) => Self::LdStVx(x as u8),
            (0xF, x, 0x1, 0xE) => Self::AddIVx(x as u8),
            (0xF, x, 0x2, 0x9) => Self::LdFVx(x as u8),
            (0xF, x, 0x3, 0x3) => Self::LdBVx(x as u8),
            (0xF, x, 0x5, 0x5) => Self::LdIVx(x as u8),
            (0xF, x, 0x6, 0x5) => Self::LdVxI(x as u8),
            _ => return Err(format!("Failed to parse instruction {value:02X}")),
        })
    }
}
