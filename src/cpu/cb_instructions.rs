// CB-Prefixed Instructions
//
// These are accessed via the 0xCB prefix opcode.
// They include rotate, shift, and bit manipulation instructions.
//
// Opcode format: 0xCB XX
// Where XX encodes the operation and target register:
//   Bits 7-6: Operation type (00=rotate/shift, 01=BIT, 10=RES, 11=SET)
//   Bits 5-3: Bit number (for BIT/RES/SET) or sub-operation (for rotate/shift)
//   Bits 2-0: Register (B=0, C=1, D=2, E=3, H=4, L=5, (HL)=6, A=7)

use super::Cpu;
use crate::bus::Bus;

impl Cpu {
    /// Execute a CB-prefixed instruction
    pub(super) fn execute_cb(&mut self, bus: &mut Bus, opcode: u8) -> u32 {
        // Extract register index (bits 2-0)
        let reg_idx = opcode & 0x07;

        // Get the value from the register (or memory at HL)
        let value = self.get_reg_value(bus, reg_idx);

        // Determine operation and execute
        let (result, cycles) = match opcode {
            // ========== RLC (Rotate Left Circular) ==========
            0x00..=0x07 => {
                let r = self.rlc(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== RRC (Rotate Right Circular) ==========
            0x08..=0x0F => {
                let r = self.rrc(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== RL (Rotate Left through Carry) ==========
            0x10..=0x17 => {
                let r = self.rl(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== RR (Rotate Right through Carry) ==========
            0x18..=0x1F => {
                let r = self.rr(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== SLA (Shift Left Arithmetic) ==========
            0x20..=0x27 => {
                let r = self.sla(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== SRA (Shift Right Arithmetic) ==========
            0x28..=0x2F => {
                let r = self.sra(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== SWAP (Swap nibbles) ==========
            0x30..=0x37 => {
                let r = self.swap(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== SRL (Shift Right Logical) ==========
            0x38..=0x3F => {
                let r = self.srl(value);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== BIT (Test bit) ==========
            0x40..=0x7F => {
                let bit = (opcode >> 3) & 0x07;
                self.bit(value, bit);
                (None, if reg_idx == 6 { 12 } else { 8 })  // BIT doesn't write back
            }

            // ========== RES (Reset bit) ==========
            0x80..=0xBF => {
                let bit = (opcode >> 3) & 0x07;
                let r = self.res(value, bit);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }

            // ========== SET (Set bit) ==========
            0xC0..=0xFF => {
                let bit = (opcode >> 3) & 0x07;
                let r = self.set(value, bit);
                (Some(r), if reg_idx == 6 { 16 } else { 8 })
            }
        };

        // Write result back to register (if applicable)
        if let Some(r) = result {
            self.set_reg_value(bus, reg_idx, r);
        }

        cycles
    }

    /// Get value from register by index
    fn get_reg_value(&self, bus: &Bus, idx: u8) -> u8 {
        match idx {
            0 => self.regs.b,
            1 => self.regs.c,
            2 => self.regs.d,
            3 => self.regs.e,
            4 => self.regs.h,
            5 => self.regs.l,
            6 => bus.read(self.regs.hl()),  // (HL)
            7 => self.regs.a,
            _ => unreachable!(),
        }
    }

    /// Set value to register by index
    fn set_reg_value(&mut self, bus: &mut Bus, idx: u8, value: u8) {
        match idx {
            0 => self.regs.b = value,
            1 => self.regs.c = value,
            2 => self.regs.d = value,
            3 => self.regs.e = value,
            4 => self.regs.h = value,
            5 => self.regs.l = value,
            6 => bus.write(self.regs.hl(), value),  // (HL)
            7 => self.regs.a = value,
            _ => unreachable!(),
        }
    }

    // ========== CB Instruction Implementations ==========

    /// RLC - Rotate Left Circular
    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = (value << 1) | carry;
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
        result
    }

    /// RRC - Rotate Right Circular
    fn rrc(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (carry << 7);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
        result
    }

    /// RL - Rotate Left through Carry
    fn rl(&mut self, value: u8) -> u8 {
        let old_carry = if self.regs.f.c { 1 } else { 0 };
        let new_carry = (value >> 7) & 1;
        let result = (value << 1) | old_carry;
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = new_carry != 0;
        result
    }

    /// RR - Rotate Right through Carry
    fn rr(&mut self, value: u8) -> u8 {
        let old_carry = if self.regs.f.c { 0x80 } else { 0 };
        let new_carry = value & 1;
        let result = (value >> 1) | old_carry;
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = new_carry != 0;
        result
    }

    /// SLA - Shift Left Arithmetic (bit 7 to carry, 0 to bit 0)
    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = value << 1;
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
        result
    }

    /// SRA - Shift Right Arithmetic (bit 0 to carry, bit 7 stays)
    fn sra(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (value & 0x80);  // Keep bit 7
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
        result
    }

    /// SWAP - Swap upper and lower nibbles
    fn swap(&mut self, value: u8) -> u8 {
        let result = ((value & 0x0F) << 4) | ((value & 0xF0) >> 4);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = false;
        result
    }

    /// SRL - Shift Right Logical (bit 0 to carry, 0 to bit 7)
    fn srl(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = value >> 1;
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
        result
    }

    /// BIT - Test bit (set Z flag if bit is 0)
    fn bit(&mut self, value: u8, bit: u8) {
        let result = value & (1 << bit);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = true;
        // C flag not affected
    }

    /// RES - Reset bit (set to 0)
    fn res(&self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    /// SET - Set bit (set to 1)
    fn set(&self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Cpu, Bus) {
        let mut cpu = Cpu::new();
        cpu.regs.pc = 0xC000;
        let bus = Bus::new();
        (cpu, bus)
    }

    #[test]
    fn test_rlc_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x85;  // 1000_0101
        bus.write(0xC000, 0xCB);  // CB prefix
        bus.write(0xC001, 0x00);  // RLC B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x0B);  // 0000_1011
        assert!(cpu.regs.f.c);  // bit 7 was set
        assert!(!cpu.regs.f.z);
    }

    #[test]
    fn test_rrc_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x01;  // 0000_0001
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x08);  // RRC B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x80);  // 1000_0000
        assert!(cpu.regs.f.c);
    }

    #[test]
    fn test_sla_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x80;  // 1000_0000
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x20);  // SLA B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x00);
        assert!(cpu.regs.f.c);  // bit 7 went to carry
        assert!(cpu.regs.f.z);  // result is zero
    }

    #[test]
    fn test_sra_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x81;  // 1000_0001
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x28);  // SRA B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0xC0);  // 1100_0000 (bit 7 preserved)
        assert!(cpu.regs.f.c);
    }

    #[test]
    fn test_swap_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0xF0;
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x30);  // SWAP B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x0F);
        assert!(!cpu.regs.f.z);
        assert!(!cpu.regs.f.c);
    }

    #[test]
    fn test_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x80;  // bit 7 set
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x78);  // BIT 7, B

        cpu.step(&mut bus);

        assert!(!cpu.regs.f.z);  // bit 7 is set
        assert!(!cpu.regs.f.n);
        assert!(cpu.regs.f.h);

        // Test bit 0 (not set)
        cpu.regs.pc = 0xC000;
        bus.write(0xC001, 0x40);  // BIT 0, B
        cpu.step(&mut bus);
        assert!(cpu.regs.f.z);  // bit 0 is not set
    }

    #[test]
    fn test_res() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0xFF;
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x80);  // RES 0, B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0xFE);
    }

    #[test]
    fn test_set() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x00;
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0xF8);  // SET 7, B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x80);
    }

    #[test]
    fn test_rl_through_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x80;
        cpu.regs.f.c = true;  // Carry set
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x10);  // RL B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x01);  // Carry rotated in
        assert!(cpu.regs.f.c);  // bit 7 went to carry
    }

    #[test]
    fn test_srl_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x81;  // 1000_0001
        bus.write(0xC000, 0xCB);
        bus.write(0xC001, 0x38);  // SRL B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x40);  // 0100_0000 (logical shift, 0 into bit 7)
        assert!(cpu.regs.f.c);  // bit 0 went to carry
    }
}
