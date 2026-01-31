// Game Boy CPU (Sharp SM83 / LR35902)
//
// This is an 8-bit CPU similar to the Z80 but with some differences.
// Clock speed: 4.194304 MHz (4,194,304 cycles per second)
//
// Registers:
//   8-bit: A, F, B, C, D, E, H, L
//   16-bit pairs: AF, BC, DE, HL
//   16-bit: SP (Stack Pointer), PC (Program Counter)
//
// Flag Register (F) bit layout:
//   Bit 7: Z (Zero Flag) - Set if result is zero
//   Bit 6: N (Subtract Flag) - Set after subtraction
//   Bit 5: H (Half Carry Flag) - Carry from bit 3 to bit 4
//   Bit 4: C (Carry Flag) - Carry from bit 7
//   Bits 3-0: Always 0

mod cb_instructions;
mod instructions;
mod registers;

pub use registers::Registers;

/// The Game Boy CPU
pub struct Cpu {
    /// CPU registers
    pub regs: Registers,
    /// Halted state - CPU stops executing until interrupt
    pub halted: bool,
    /// Interrupt Master Enable flag
    pub ime: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            halted: false,
            ime: false,
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        // After boot ROM, PC should be at 0x0100
        // But we initialize to 0 and will set properly during boot
        assert_eq!(cpu.regs.pc, 0x0100);
        assert_eq!(cpu.regs.sp, 0xFFFE);
        assert!(!cpu.halted);
        assert!(!cpu.ime);
    }
}
