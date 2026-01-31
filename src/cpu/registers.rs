// CPU Registers
//
// The Game Boy CPU has the following registers:
//
// 8-bit registers: A, F, B, C, D, E, H, L
// These can be paired into 16-bit registers: AF, BC, DE, HL
//
// A (Accumulator): Main register for arithmetic operations
// F (Flags): Contains CPU flags (Z, N, H, C)
// B, C, D, E, H, L: General purpose registers
//
// Special 16-bit registers:
// SP (Stack Pointer): Points to current stack position
// PC (Program Counter): Address of next instruction
//
// Flag Register (F) layout:
// Bit 7 6 5 4 3 2 1 0
//     Z N H C 0 0 0 0
//
// Z (Zero): Set when result is 0
// N (Subtract): Set after subtraction instructions
// H (Half Carry): Set when carry from bit 3 to 4 (for BCD)
// C (Carry): Set when carry from bit 7 (overflow)

/// CPU Flag bits
#[derive(Debug, Clone, Copy)]
pub struct Flags {
    /// Zero flag - set when result is zero
    pub z: bool,
    /// Subtract flag - set after subtraction
    pub n: bool,
    /// Half carry flag - carry from bit 3 to bit 4
    pub h: bool,
    /// Carry flag - carry from bit 7
    pub c: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            z: true,  // Post-boot value
            n: false,
            h: true,  // Post-boot value
            c: true,  // Post-boot value
        }
    }

    /// Convert flags to the F register byte
    pub fn to_byte(&self) -> u8 {
        let mut f = 0u8;
        if self.z { f |= 0x80; }  // Bit 7
        if self.n { f |= 0x40; }  // Bit 6
        if self.h { f |= 0x20; }  // Bit 5
        if self.c { f |= 0x10; }  // Bit 4
        f
    }

    /// Set flags from F register byte
    pub fn from_byte(&mut self, byte: u8) {
        self.z = (byte & 0x80) != 0;
        self.n = (byte & 0x40) != 0;
        self.h = (byte & 0x20) != 0;
        self.c = (byte & 0x10) != 0;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Registers
#[derive(Debug, Clone)]
pub struct Registers {
    /// Accumulator
    pub a: u8,
    /// Flags
    pub f: Flags,
    /// General purpose register B
    pub b: u8,
    /// General purpose register C
    pub c: u8,
    /// General purpose register D
    pub d: u8,
    /// General purpose register E
    pub e: u8,
    /// General purpose register H
    pub h: u8,
    /// General purpose register L
    pub l: u8,
    /// Stack Pointer
    pub sp: u16,
    /// Program Counter
    pub pc: u16,
}

impl Registers {
    /// Create new registers with post-boot ROM values
    /// These values are what the CPU has after the boot ROM finishes
    /// Reference: Pan Docs - Power Up Sequence
    pub fn new() -> Self {
        Self {
            a: 0x01,   // Post-boot value (DMG)
            f: Flags::new(),
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100, // Entry point after boot ROM
        }
    }

    // 16-bit register pair accessors
    // AF, BC, DE, HL combine two 8-bit registers into one 16-bit value
    // High byte comes first (e.g., A is high byte of AF)

    /// Get AF register pair (A << 8 | F)
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.to_byte() as u16)
    }

    /// Set AF register pair
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f.from_byte((value & 0xF0) as u8); // Lower 4 bits always 0
    }

    /// Get BC register pair
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Set BC register pair
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    /// Get DE register pair
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Set DE register pair
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    /// Get HL register pair
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Set HL register pair
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_to_byte() {
        let mut flags = Flags { z: false, n: false, h: false, c: false };
        assert_eq!(flags.to_byte(), 0x00);

        flags.z = true;
        assert_eq!(flags.to_byte(), 0x80);

        flags.n = true;
        assert_eq!(flags.to_byte(), 0xC0);

        flags.h = true;
        assert_eq!(flags.to_byte(), 0xE0);

        flags.c = true;
        assert_eq!(flags.to_byte(), 0xF0);
    }

    #[test]
    fn test_flags_from_byte() {
        let mut flags = Flags::new();

        flags.from_byte(0x00);
        assert!(!flags.z && !flags.n && !flags.h && !flags.c);

        flags.from_byte(0xF0);
        assert!(flags.z && flags.n && flags.h && flags.c);

        // Lower 4 bits should be ignored
        flags.from_byte(0xFF);
        assert!(flags.z && flags.n && flags.h && flags.c);
    }

    #[test]
    fn test_register_pairs() {
        let mut regs = Registers::new();

        // Test BC
        regs.set_bc(0x1234);
        assert_eq!(regs.b, 0x12);
        assert_eq!(regs.c, 0x34);
        assert_eq!(regs.bc(), 0x1234);

        // Test DE
        regs.set_de(0xABCD);
        assert_eq!(regs.d, 0xAB);
        assert_eq!(regs.e, 0xCD);
        assert_eq!(regs.de(), 0xABCD);

        // Test HL
        regs.set_hl(0x5678);
        assert_eq!(regs.h, 0x56);
        assert_eq!(regs.l, 0x78);
        assert_eq!(regs.hl(), 0x5678);
    }

    #[test]
    fn test_af_pair() {
        let mut regs = Registers::new();

        // Set AF with specific flags
        regs.set_af(0x12F0); // A=0x12, F=0xF0 (all flags set)
        assert_eq!(regs.a, 0x12);
        assert!(regs.f.z && regs.f.n && regs.f.h && regs.f.c);

        // Lower 4 bits of F should be masked
        regs.set_af(0x34FF);
        assert_eq!(regs.a, 0x34);
        assert_eq!(regs.af() & 0x00FF, 0xF0); // Lower 4 bits always 0
    }

    #[test]
    fn test_post_boot_values() {
        let regs = Registers::new();

        // DMG post-boot register values
        assert_eq!(regs.a, 0x01);
        assert_eq!(regs.b, 0x00);
        assert_eq!(regs.c, 0x13);
        assert_eq!(regs.d, 0x00);
        assert_eq!(regs.e, 0xD8);
        assert_eq!(regs.h, 0x01);
        assert_eq!(regs.l, 0x4D);
        assert_eq!(regs.sp, 0xFFFE);
        assert_eq!(regs.pc, 0x0100);
    }
}
