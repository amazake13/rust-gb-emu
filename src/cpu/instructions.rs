// CPU Instructions
//
// The SM83 CPU has 256 base opcodes (0x00-0xFF) plus 256 CB-prefixed opcodes.
// Each instruction takes a certain number of machine cycles (M-cycles).
// 1 M-cycle = 4 T-cycles (clock cycles)
//
// Instruction timing:
// - Most instructions take 1-6 M-cycles
// - Memory access takes 1 M-cycle per byte
// - Conditional branches may take different times depending on condition

use super::Cpu;
use crate::bus::Bus;

impl Cpu {
    /// Fetch, decode, and execute one instruction
    /// Returns the number of T-cycles (clock cycles) consumed
    pub fn step(&mut self, bus: &mut Bus) -> u32 {
        // Handle pending interrupts first
        let interrupt_cycles = self.handle_interrupts(bus);
        if interrupt_cycles > 0 {
            return interrupt_cycles;
        }

        if self.halted {
            // HALT mode: CPU waits for interrupt
            // Still consume cycles
            return 4;
        }

        // Remember if EI was scheduled before this instruction
        let ei_pending = self.ime_scheduled;

        let opcode = self.fetch(bus);
        let cycles = self.execute(bus, opcode);

        // Apply scheduled IME enable AFTER the instruction executes
        // (EI has 1 instruction delay)
        if ei_pending {
            self.ime = true;
            self.ime_scheduled = false;
        }

        cycles
    }

    /// Fetch the next byte from PC and increment PC
    fn fetch(&mut self, bus: &Bus) -> u8 {
        let byte = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }

    /// Fetch a 16-bit value (little-endian)
    fn fetch16(&mut self, bus: &Bus) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        (hi << 8) | lo
    }

    /// Execute an instruction and return cycles consumed
    fn execute(&mut self, bus: &mut Bus, opcode: u8) -> u32 {
        match opcode {
            // ========== NOP ==========
            // 0x00: NOP - No operation
            0x00 => 4,

            // ========== STOP ==========
            // 0x10: STOP - Halt CPU & LCD until button pressed
            // In practice, often used as a 2-byte NOP (0x10 0x00)
            0x10 => {
                self.fetch(bus); // Consume the next byte (usually 0x00)
                // For now, treat as NOP. Real STOP would halt until joypad input.
                4
            }

            // ========== LD r, n (8-bit immediate) ==========
            // Load 8-bit immediate value into register
            0x06 => { self.regs.b = self.fetch(bus); 8 }  // LD B, n
            0x0E => { self.regs.c = self.fetch(bus); 8 }  // LD C, n
            0x16 => { self.regs.d = self.fetch(bus); 8 }  // LD D, n
            0x1E => { self.regs.e = self.fetch(bus); 8 }  // LD E, n
            0x26 => { self.regs.h = self.fetch(bus); 8 }  // LD H, n
            0x2E => { self.regs.l = self.fetch(bus); 8 }  // LD L, n
            0x3E => { self.regs.a = self.fetch(bus); 8 }  // LD A, n

            // ========== LD r, r (8-bit register to register) ==========
            // LD B, r
            0x40 => 4,  // LD B, B
            0x41 => { self.regs.b = self.regs.c; 4 }
            0x42 => { self.regs.b = self.regs.d; 4 }
            0x43 => { self.regs.b = self.regs.e; 4 }
            0x44 => { self.regs.b = self.regs.h; 4 }
            0x45 => { self.regs.b = self.regs.l; 4 }
            0x46 => { self.regs.b = bus.read(self.regs.hl()); 8 }  // LD B, (HL)
            0x47 => { self.regs.b = self.regs.a; 4 }

            // LD C, r
            0x48 => { self.regs.c = self.regs.b; 4 }
            0x49 => 4,  // LD C, C
            0x4A => { self.regs.c = self.regs.d; 4 }
            0x4B => { self.regs.c = self.regs.e; 4 }
            0x4C => { self.regs.c = self.regs.h; 4 }
            0x4D => { self.regs.c = self.regs.l; 4 }
            0x4E => { self.regs.c = bus.read(self.regs.hl()); 8 }
            0x4F => { self.regs.c = self.regs.a; 4 }

            // LD D, r
            0x50 => { self.regs.d = self.regs.b; 4 }
            0x51 => { self.regs.d = self.regs.c; 4 }
            0x52 => 4,  // LD D, D
            0x53 => { self.regs.d = self.regs.e; 4 }
            0x54 => { self.regs.d = self.regs.h; 4 }
            0x55 => { self.regs.d = self.regs.l; 4 }
            0x56 => { self.regs.d = bus.read(self.regs.hl()); 8 }
            0x57 => { self.regs.d = self.regs.a; 4 }

            // LD E, r
            0x58 => { self.regs.e = self.regs.b; 4 }
            0x59 => { self.regs.e = self.regs.c; 4 }
            0x5A => { self.regs.e = self.regs.d; 4 }
            0x5B => 4,  // LD E, E
            0x5C => { self.regs.e = self.regs.h; 4 }
            0x5D => { self.regs.e = self.regs.l; 4 }
            0x5E => { self.regs.e = bus.read(self.regs.hl()); 8 }
            0x5F => { self.regs.e = self.regs.a; 4 }

            // LD H, r
            0x60 => { self.regs.h = self.regs.b; 4 }
            0x61 => { self.regs.h = self.regs.c; 4 }
            0x62 => { self.regs.h = self.regs.d; 4 }
            0x63 => { self.regs.h = self.regs.e; 4 }
            0x64 => 4,  // LD H, H
            0x65 => { self.regs.h = self.regs.l; 4 }
            0x66 => { self.regs.h = bus.read(self.regs.hl()); 8 }
            0x67 => { self.regs.h = self.regs.a; 4 }

            // LD L, r
            0x68 => { self.regs.l = self.regs.b; 4 }
            0x69 => { self.regs.l = self.regs.c; 4 }
            0x6A => { self.regs.l = self.regs.d; 4 }
            0x6B => { self.regs.l = self.regs.e; 4 }
            0x6C => { self.regs.l = self.regs.h; 4 }
            0x6D => 4,  // LD L, L
            0x6E => { self.regs.l = bus.read(self.regs.hl()); 8 }
            0x6F => { self.regs.l = self.regs.a; 4 }

            // LD (HL), r
            0x70 => { bus.write(self.regs.hl(), self.regs.b); 8 }
            0x71 => { bus.write(self.regs.hl(), self.regs.c); 8 }
            0x72 => { bus.write(self.regs.hl(), self.regs.d); 8 }
            0x73 => { bus.write(self.regs.hl(), self.regs.e); 8 }
            0x74 => { bus.write(self.regs.hl(), self.regs.h); 8 }
            0x75 => { bus.write(self.regs.hl(), self.regs.l); 8 }
            // 0x76 is HALT
            0x77 => { bus.write(self.regs.hl(), self.regs.a); 8 }

            // LD A, r
            0x78 => { self.regs.a = self.regs.b; 4 }
            0x79 => { self.regs.a = self.regs.c; 4 }
            0x7A => { self.regs.a = self.regs.d; 4 }
            0x7B => { self.regs.a = self.regs.e; 4 }
            0x7C => { self.regs.a = self.regs.h; 4 }
            0x7D => { self.regs.a = self.regs.l; 4 }
            0x7E => { self.regs.a = bus.read(self.regs.hl()); 8 }
            0x7F => 4,  // LD A, A

            // ========== LD rr, nn (16-bit immediate) ==========
            0x01 => { let v = self.fetch16(bus); self.regs.set_bc(v); 12 }  // LD BC, nn
            0x11 => { let v = self.fetch16(bus); self.regs.set_de(v); 12 }  // LD DE, nn
            0x21 => { let v = self.fetch16(bus); self.regs.set_hl(v); 12 }  // LD HL, nn
            0x31 => { self.regs.sp = self.fetch16(bus); 12 }                 // LD SP, nn

            // ========== LD A, (rr) / LD (rr), A ==========
            0x02 => { bus.write(self.regs.bc(), self.regs.a); 8 }  // LD (BC), A
            0x12 => { bus.write(self.regs.de(), self.regs.a); 8 }  // LD (DE), A
            0x0A => { self.regs.a = bus.read(self.regs.bc()); 8 }  // LD A, (BC)
            0x1A => { self.regs.a = bus.read(self.regs.de()); 8 }  // LD A, (DE)

            // LD A, (HL+) / LD A, (HL-) / LD (HL+), A / LD (HL-), A
            0x22 => {  // LD (HL+), A
                bus.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
                8
            }
            0x32 => {  // LD (HL-), A
                bus.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                8
            }
            0x2A => {  // LD A, (HL+)
                self.regs.a = bus.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
                8
            }
            0x3A => {  // LD A, (HL-)
                self.regs.a = bus.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                8
            }

            // LD (nn), A / LD A, (nn)
            0xEA => {  // LD (nn), A
                let addr = self.fetch16(bus);
                bus.write(addr, self.regs.a);
                16
            }
            0xFA => {  // LD A, (nn)
                let addr = self.fetch16(bus);
                self.regs.a = bus.read(addr);
                16
            }

            // LDH (n), A / LDH A, (n) - High RAM access
            0xE0 => {  // LDH (n), A - LD (0xFF00+n), A
                let offset = self.fetch(bus) as u16;
                bus.write(0xFF00 + offset, self.regs.a);
                12
            }
            0xF0 => {  // LDH A, (n) - LD A, (0xFF00+n)
                let offset = self.fetch(bus) as u16;
                self.regs.a = bus.read(0xFF00 + offset);
                12
            }

            // LDH (C), A / LDH A, (C)
            0xE2 => {  // LD (0xFF00+C), A
                bus.write(0xFF00 + self.regs.c as u16, self.regs.a);
                8
            }
            0xF2 => {  // LD A, (0xFF00+C)
                self.regs.a = bus.read(0xFF00 + self.regs.c as u16);
                8
            }

            // LD (HL), n
            0x36 => {
                let n = self.fetch(bus);
                bus.write(self.regs.hl(), n);
                12
            }

            // LD SP, HL
            0xF9 => { self.regs.sp = self.regs.hl(); 8 }

            // LD (nn), SP
            0x08 => {
                let addr = self.fetch16(bus);
                bus.write16(addr, self.regs.sp);
                20
            }

            // ========== INC/DEC 8-bit ==========
            0x04 => { self.regs.b = self.inc(self.regs.b); 4 }  // INC B
            0x0C => { self.regs.c = self.inc(self.regs.c); 4 }  // INC C
            0x14 => { self.regs.d = self.inc(self.regs.d); 4 }  // INC D
            0x1C => { self.regs.e = self.inc(self.regs.e); 4 }  // INC E
            0x24 => { self.regs.h = self.inc(self.regs.h); 4 }  // INC H
            0x2C => { self.regs.l = self.inc(self.regs.l); 4 }  // INC L
            0x34 => {  // INC (HL)
                let v = self.inc(bus.read(self.regs.hl()));
                bus.write(self.regs.hl(), v);
                12
            }
            0x3C => { self.regs.a = self.inc(self.regs.a); 4 }  // INC A

            0x05 => { self.regs.b = self.dec(self.regs.b); 4 }  // DEC B
            0x0D => { self.regs.c = self.dec(self.regs.c); 4 }  // DEC C
            0x15 => { self.regs.d = self.dec(self.regs.d); 4 }  // DEC D
            0x1D => { self.regs.e = self.dec(self.regs.e); 4 }  // DEC E
            0x25 => { self.regs.h = self.dec(self.regs.h); 4 }  // DEC H
            0x2D => { self.regs.l = self.dec(self.regs.l); 4 }  // DEC L
            0x35 => {  // DEC (HL)
                let v = self.dec(bus.read(self.regs.hl()));
                bus.write(self.regs.hl(), v);
                12
            }
            0x3D => { self.regs.a = self.dec(self.regs.a); 4 }  // DEC A

            // ========== INC/DEC 16-bit ==========
            0x03 => { self.regs.set_bc(self.regs.bc().wrapping_add(1)); 8 }  // INC BC
            0x13 => { self.regs.set_de(self.regs.de().wrapping_add(1)); 8 }  // INC DE
            0x23 => { self.regs.set_hl(self.regs.hl().wrapping_add(1)); 8 }  // INC HL
            0x33 => { self.regs.sp = self.regs.sp.wrapping_add(1); 8 }       // INC SP

            0x0B => { self.regs.set_bc(self.regs.bc().wrapping_sub(1)); 8 }  // DEC BC
            0x1B => { self.regs.set_de(self.regs.de().wrapping_sub(1)); 8 }  // DEC DE
            0x2B => { self.regs.set_hl(self.regs.hl().wrapping_sub(1)); 8 }  // DEC HL
            0x3B => { self.regs.sp = self.regs.sp.wrapping_sub(1); 8 }       // DEC SP

            // ========== ADD A, r ==========
            0x80 => { self.add(self.regs.b); 4 }
            0x81 => { self.add(self.regs.c); 4 }
            0x82 => { self.add(self.regs.d); 4 }
            0x83 => { self.add(self.regs.e); 4 }
            0x84 => { self.add(self.regs.h); 4 }
            0x85 => { self.add(self.regs.l); 4 }
            0x86 => { self.add(bus.read(self.regs.hl())); 8 }
            0x87 => { self.add(self.regs.a); 4 }
            0xC6 => { let n = self.fetch(bus); self.add(n); 8 }  // ADD A, n

            // ========== ADC A, r (Add with Carry) ==========
            0x88 => { self.adc(self.regs.b); 4 }
            0x89 => { self.adc(self.regs.c); 4 }
            0x8A => { self.adc(self.regs.d); 4 }
            0x8B => { self.adc(self.regs.e); 4 }
            0x8C => { self.adc(self.regs.h); 4 }
            0x8D => { self.adc(self.regs.l); 4 }
            0x8E => { self.adc(bus.read(self.regs.hl())); 8 }
            0x8F => { self.adc(self.regs.a); 4 }
            0xCE => { let n = self.fetch(bus); self.adc(n); 8 }  // ADC A, n

            // ========== SUB A, r ==========
            0x90 => { self.sub(self.regs.b); 4 }
            0x91 => { self.sub(self.regs.c); 4 }
            0x92 => { self.sub(self.regs.d); 4 }
            0x93 => { self.sub(self.regs.e); 4 }
            0x94 => { self.sub(self.regs.h); 4 }
            0x95 => { self.sub(self.regs.l); 4 }
            0x96 => { self.sub(bus.read(self.regs.hl())); 8 }
            0x97 => { self.sub(self.regs.a); 4 }
            0xD6 => { let n = self.fetch(bus); self.sub(n); 8 }  // SUB n

            // ========== SBC A, r (Subtract with Carry) ==========
            0x98 => { self.sbc(self.regs.b); 4 }
            0x99 => { self.sbc(self.regs.c); 4 }
            0x9A => { self.sbc(self.regs.d); 4 }
            0x9B => { self.sbc(self.regs.e); 4 }
            0x9C => { self.sbc(self.regs.h); 4 }
            0x9D => { self.sbc(self.regs.l); 4 }
            0x9E => { self.sbc(bus.read(self.regs.hl())); 8 }
            0x9F => { self.sbc(self.regs.a); 4 }
            0xDE => { let n = self.fetch(bus); self.sbc(n); 8 }  // SBC A, n

            // ========== AND A, r ==========
            0xA0 => { self.and(self.regs.b); 4 }
            0xA1 => { self.and(self.regs.c); 4 }
            0xA2 => { self.and(self.regs.d); 4 }
            0xA3 => { self.and(self.regs.e); 4 }
            0xA4 => { self.and(self.regs.h); 4 }
            0xA5 => { self.and(self.regs.l); 4 }
            0xA6 => { self.and(bus.read(self.regs.hl())); 8 }
            0xA7 => { self.and(self.regs.a); 4 }
            0xE6 => { let n = self.fetch(bus); self.and(n); 8 }  // AND n

            // ========== XOR A, r ==========
            0xA8 => { self.xor(self.regs.b); 4 }
            0xA9 => { self.xor(self.regs.c); 4 }
            0xAA => { self.xor(self.regs.d); 4 }
            0xAB => { self.xor(self.regs.e); 4 }
            0xAC => { self.xor(self.regs.h); 4 }
            0xAD => { self.xor(self.regs.l); 4 }
            0xAE => { self.xor(bus.read(self.regs.hl())); 8 }
            0xAF => { self.xor(self.regs.a); 4 }
            0xEE => { let n = self.fetch(bus); self.xor(n); 8 }  // XOR n

            // ========== OR A, r ==========
            0xB0 => { self.or(self.regs.b); 4 }
            0xB1 => { self.or(self.regs.c); 4 }
            0xB2 => { self.or(self.regs.d); 4 }
            0xB3 => { self.or(self.regs.e); 4 }
            0xB4 => { self.or(self.regs.h); 4 }
            0xB5 => { self.or(self.regs.l); 4 }
            0xB6 => { self.or(bus.read(self.regs.hl())); 8 }
            0xB7 => { self.or(self.regs.a); 4 }
            0xF6 => { let n = self.fetch(bus); self.or(n); 8 }  // OR n

            // ========== CP A, r (Compare) ==========
            0xB8 => { self.cp(self.regs.b); 4 }
            0xB9 => { self.cp(self.regs.c); 4 }
            0xBA => { self.cp(self.regs.d); 4 }
            0xBB => { self.cp(self.regs.e); 4 }
            0xBC => { self.cp(self.regs.h); 4 }
            0xBD => { self.cp(self.regs.l); 4 }
            0xBE => { self.cp(bus.read(self.regs.hl())); 8 }
            0xBF => { self.cp(self.regs.a); 4 }
            0xFE => { let n = self.fetch(bus); self.cp(n); 8 }  // CP n

            // ========== ADD HL, rr (16-bit add) ==========
            0x09 => { self.add_hl(self.regs.bc()); 8 }  // ADD HL, BC
            0x19 => { self.add_hl(self.regs.de()); 8 }  // ADD HL, DE
            0x29 => { self.add_hl(self.regs.hl()); 8 }  // ADD HL, HL
            0x39 => { self.add_hl(self.regs.sp); 8 }    // ADD HL, SP

            // ========== JP (Jump) ==========
            0xC3 => { self.regs.pc = self.fetch16(bus); 16 }  // JP nn
            0xE9 => { self.regs.pc = self.regs.hl(); 4 }      // JP HL

            // Conditional jumps
            0xC2 => {  // JP NZ, nn
                let addr = self.fetch16(bus);
                if !self.regs.f.z { self.regs.pc = addr; 16 } else { 12 }
            }
            0xCA => {  // JP Z, nn
                let addr = self.fetch16(bus);
                if self.regs.f.z { self.regs.pc = addr; 16 } else { 12 }
            }
            0xD2 => {  // JP NC, nn
                let addr = self.fetch16(bus);
                if !self.regs.f.c { self.regs.pc = addr; 16 } else { 12 }
            }
            0xDA => {  // JP C, nn
                let addr = self.fetch16(bus);
                if self.regs.f.c { self.regs.pc = addr; 16 } else { 12 }
            }

            // ========== JR (Relative Jump) ==========
            0x18 => {  // JR n
                let offset = self.fetch(bus) as i8;
                self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                12
            }
            0x20 => {  // JR NZ, n
                let offset = self.fetch(bus) as i8;
                if !self.regs.f.z {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else { 8 }
            }
            0x28 => {  // JR Z, n
                let offset = self.fetch(bus) as i8;
                if self.regs.f.z {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else { 8 }
            }
            0x30 => {  // JR NC, n
                let offset = self.fetch(bus) as i8;
                if !self.regs.f.c {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else { 8 }
            }
            0x38 => {  // JR C, n
                let offset = self.fetch(bus) as i8;
                if self.regs.f.c {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else { 8 }
            }

            // ========== CALL ==========
            0xCD => {  // CALL nn
                let addr = self.fetch16(bus);
                self.push(bus, self.regs.pc);
                self.regs.pc = addr;
                24
            }
            0xC4 => {  // CALL NZ, nn
                let addr = self.fetch16(bus);
                if !self.regs.f.z { self.push(bus, self.regs.pc); self.regs.pc = addr; 24 } else { 12 }
            }
            0xCC => {  // CALL Z, nn
                let addr = self.fetch16(bus);
                if self.regs.f.z { self.push(bus, self.regs.pc); self.regs.pc = addr; 24 } else { 12 }
            }
            0xD4 => {  // CALL NC, nn
                let addr = self.fetch16(bus);
                if !self.regs.f.c { self.push(bus, self.regs.pc); self.regs.pc = addr; 24 } else { 12 }
            }
            0xDC => {  // CALL C, nn
                let addr = self.fetch16(bus);
                if self.regs.f.c { self.push(bus, self.regs.pc); self.regs.pc = addr; 24 } else { 12 }
            }

            // ========== RET ==========
            0xC9 => { self.regs.pc = self.pop(bus); 16 }  // RET
            0xD9 => {  // RETI
                self.regs.pc = self.pop(bus);
                self.ime = true;
                16
            }
            0xC0 => { if !self.regs.f.z { self.regs.pc = self.pop(bus); 20 } else { 8 } }  // RET NZ
            0xC8 => { if self.regs.f.z { self.regs.pc = self.pop(bus); 20 } else { 8 } }   // RET Z
            0xD0 => { if !self.regs.f.c { self.regs.pc = self.pop(bus); 20 } else { 8 } }  // RET NC
            0xD8 => { if self.regs.f.c { self.regs.pc = self.pop(bus); 20 } else { 8 } }   // RET C

            // ========== RST (Restart) ==========
            0xC7 => { self.push(bus, self.regs.pc); self.regs.pc = 0x00; 16 }  // RST 00H
            0xCF => { self.push(bus, self.regs.pc); self.regs.pc = 0x08; 16 }  // RST 08H
            0xD7 => { self.push(bus, self.regs.pc); self.regs.pc = 0x10; 16 }  // RST 10H
            0xDF => { self.push(bus, self.regs.pc); self.regs.pc = 0x18; 16 }  // RST 18H
            0xE7 => { self.push(bus, self.regs.pc); self.regs.pc = 0x20; 16 }  // RST 20H
            0xEF => { self.push(bus, self.regs.pc); self.regs.pc = 0x28; 16 }  // RST 28H
            0xF7 => { self.push(bus, self.regs.pc); self.regs.pc = 0x30; 16 }  // RST 30H
            0xFF => { self.push(bus, self.regs.pc); self.regs.pc = 0x38; 16 }  // RST 38H

            // ========== PUSH/POP ==========
            0xC5 => { self.push(bus, self.regs.bc()); 16 }  // PUSH BC
            0xD5 => { self.push(bus, self.regs.de()); 16 }  // PUSH DE
            0xE5 => { self.push(bus, self.regs.hl()); 16 }  // PUSH HL
            0xF5 => { self.push(bus, self.regs.af()); 16 }  // PUSH AF

            0xC1 => { let v = self.pop(bus); self.regs.set_bc(v); 12 }  // POP BC
            0xD1 => { let v = self.pop(bus); self.regs.set_de(v); 12 }  // POP DE
            0xE1 => { let v = self.pop(bus); self.regs.set_hl(v); 12 }  // POP HL
            0xF1 => { let v = self.pop(bus); self.regs.set_af(v); 12 }  // POP AF

            // ========== Interrupt control ==========
            0xF3 => {  // DI (Disable Interrupts)
                self.ime = false;
                self.ime_scheduled = false;
                4
            }
            0xFB => {  // EI (Enable Interrupts)
                // EI has a 1 instruction delay - IME is set after the next instruction
                self.ime_scheduled = true;
                4
            }

            // ========== HALT ==========
            0x76 => { self.halted = true; 4 }

            // ========== Rotates and shifts ==========
            0x07 => { self.rlca(); 4 }   // RLCA
            0x0F => { self.rrca(); 4 }   // RRCA
            0x17 => { self.rla(); 4 }    // RLA
            0x1F => { self.rra(); 4 }    // RRA

            // ========== Misc ==========
            0x27 => { self.daa(); 4 }    // DAA
            0x2F => { self.cpl(); 4 }    // CPL
            0x37 => { self.scf(); 4 }    // SCF
            0x3F => { self.ccf(); 4 }    // CCF

            // ========== ADD SP, n / LD HL, SP+n ==========
            0xE8 => {  // ADD SP, n
                let n = self.fetch(bus) as i8 as i16 as u16;
                let result = self.regs.sp.wrapping_add(n);
                self.regs.f.z = false;
                self.regs.f.n = false;
                self.regs.f.h = (self.regs.sp & 0x0F) + (n & 0x0F) > 0x0F;
                self.regs.f.c = (self.regs.sp & 0xFF) + (n & 0xFF) > 0xFF;
                self.regs.sp = result;
                16
            }
            0xF8 => {  // LD HL, SP+n
                let n = self.fetch(bus) as i8 as i16 as u16;
                let result = self.regs.sp.wrapping_add(n);
                self.regs.f.z = false;
                self.regs.f.n = false;
                self.regs.f.h = (self.regs.sp & 0x0F) + (n & 0x0F) > 0x0F;
                self.regs.f.c = (self.regs.sp & 0xFF) + (n & 0xFF) > 0xFF;
                self.regs.set_hl(result);
                12
            }

            // ========== CB prefix ==========
            0xCB => {
                let cb_opcode = self.fetch(bus);
                self.execute_cb(bus, cb_opcode)
            }

            // ========== Undefined opcodes ==========
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                // These opcodes are undefined on the Game Boy
                // Real hardware behavior varies, often acts like NOP or crashes
                panic!("Undefined opcode: 0x{:02X} at 0x{:04X}", opcode, self.regs.pc.wrapping_sub(1));
            }

            // For debugging: halt on unimplemented
            _ => {
                panic!("Unimplemented opcode: 0x{:02X} at 0x{:04X}", opcode, self.regs.pc.wrapping_sub(1));
            }
        }
    }

    // ========== ALU Helper Functions ==========

    /// INC r - Increment register
    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = (value & 0x0F) + 1 > 0x0F;
        // C flag not affected
        result
    }

    /// DEC r - Decrement register
    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.regs.f.z = result == 0;
        self.regs.f.n = true;
        self.regs.f.h = (value & 0x0F) == 0;
        // C flag not affected
        result
    }

    /// ADD A, r
    fn add(&mut self, value: u8) {
        let (result, carry) = self.regs.a.overflowing_add(value);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = (self.regs.a & 0x0F) + (value & 0x0F) > 0x0F;
        self.regs.f.c = carry;
        self.regs.a = result;
    }

    /// ADC A, r (Add with Carry)
    fn adc(&mut self, value: u8) {
        let carry = if self.regs.f.c { 1u8 } else { 0u8 };
        let result = self.regs.a.wrapping_add(value).wrapping_add(carry);
        self.regs.f.z = result == 0;
        self.regs.f.n = false;
        self.regs.f.h = (self.regs.a & 0x0F) + (value & 0x0F) + carry > 0x0F;
        self.regs.f.c = (self.regs.a as u16) + (value as u16) + (carry as u16) > 0xFF;
        self.regs.a = result;
    }

    /// SUB A, r
    fn sub(&mut self, value: u8) {
        let (result, borrow) = self.regs.a.overflowing_sub(value);
        self.regs.f.z = result == 0;
        self.regs.f.n = true;
        self.regs.f.h = (self.regs.a & 0x0F) < (value & 0x0F);
        self.regs.f.c = borrow;
        self.regs.a = result;
    }

    /// SBC A, r (Subtract with Carry)
    fn sbc(&mut self, value: u8) {
        let carry = if self.regs.f.c { 1u8 } else { 0u8 };
        let result = self.regs.a.wrapping_sub(value).wrapping_sub(carry);
        self.regs.f.z = result == 0;
        self.regs.f.n = true;
        self.regs.f.h = (self.regs.a & 0x0F) < (value & 0x0F) + carry;
        self.regs.f.c = (self.regs.a as u16) < (value as u16) + (carry as u16);
        self.regs.a = result;
    }

    /// AND A, r
    fn and(&mut self, value: u8) {
        self.regs.a &= value;
        self.regs.f.z = self.regs.a == 0;
        self.regs.f.n = false;
        self.regs.f.h = true;
        self.regs.f.c = false;
    }

    /// XOR A, r
    fn xor(&mut self, value: u8) {
        self.regs.a ^= value;
        self.regs.f.z = self.regs.a == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = false;
    }

    /// OR A, r
    fn or(&mut self, value: u8) {
        self.regs.a |= value;
        self.regs.f.z = self.regs.a == 0;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = false;
    }

    /// CP A, r (Compare - like SUB but discard result)
    fn cp(&mut self, value: u8) {
        let result = self.regs.a.wrapping_sub(value);
        self.regs.f.z = result == 0;
        self.regs.f.n = true;
        self.regs.f.h = (self.regs.a & 0x0F) < (value & 0x0F);
        self.regs.f.c = self.regs.a < value;
    }

    /// ADD HL, rr (16-bit add)
    fn add_hl(&mut self, value: u16) {
        let hl = self.regs.hl();
        let (result, carry) = hl.overflowing_add(value);
        // Z flag not affected
        self.regs.f.n = false;
        self.regs.f.h = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
        self.regs.f.c = carry;
        self.regs.set_hl(result);
    }

    // ========== Stack operations ==========

    /// Push 16-bit value onto stack
    fn push(&mut self, bus: &mut Bus, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, (value >> 8) as u8);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, (value & 0xFF) as u8);
    }

    /// Pop 16-bit value from stack
    fn pop(&mut self, bus: &Bus) -> u16 {
        let lo = bus.read(self.regs.sp) as u16;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let hi = bus.read(self.regs.sp) as u16;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        (hi << 8) | lo
    }

    // ========== Rotate instructions ==========

    /// RLCA - Rotate A left (circular)
    fn rlca(&mut self) {
        let carry = (self.regs.a >> 7) & 1;
        self.regs.a = (self.regs.a << 1) | carry;
        self.regs.f.z = false;  // Always false for RLCA
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
    }

    /// RRCA - Rotate A right (circular)
    fn rrca(&mut self) {
        let carry = self.regs.a & 1;
        self.regs.a = (self.regs.a >> 1) | (carry << 7);
        self.regs.f.z = false;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = carry != 0;
    }

    /// RLA - Rotate A left through carry
    fn rla(&mut self) {
        let old_carry = if self.regs.f.c { 1 } else { 0 };
        let new_carry = (self.regs.a >> 7) & 1;
        self.regs.a = (self.regs.a << 1) | old_carry;
        self.regs.f.z = false;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = new_carry != 0;
    }

    /// RRA - Rotate A right through carry
    fn rra(&mut self) {
        let old_carry = if self.regs.f.c { 0x80 } else { 0 };
        let new_carry = self.regs.a & 1;
        self.regs.a = (self.regs.a >> 1) | old_carry;
        self.regs.f.z = false;
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = new_carry != 0;
    }

    // ========== Misc instructions ==========

    /// DAA - Decimal Adjust Accumulator
    fn daa(&mut self) {
        let mut adjust = 0u8;

        if self.regs.f.n {
            // After subtraction
            if self.regs.f.c { adjust |= 0x60; }
            if self.regs.f.h { adjust |= 0x06; }
            self.regs.a = self.regs.a.wrapping_sub(adjust);
        } else {
            // After addition
            if self.regs.f.c || self.regs.a > 0x99 {
                adjust |= 0x60;
                self.regs.f.c = true;
            }
            if self.regs.f.h || (self.regs.a & 0x0F) > 0x09 {
                adjust |= 0x06;
            }
            self.regs.a = self.regs.a.wrapping_add(adjust);
        }

        self.regs.f.z = self.regs.a == 0;
        self.regs.f.h = false;
    }

    /// CPL - Complement A (flip all bits)
    fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.f.n = true;
        self.regs.f.h = true;
    }

    /// SCF - Set Carry Flag
    fn scf(&mut self) {
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = true;
    }

    /// CCF - Complement Carry Flag
    fn ccf(&mut self) {
        self.regs.f.n = false;
        self.regs.f.h = false;
        self.regs.f.c = !self.regs.f.c;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Cpu, Bus) {
        let mut cpu = Cpu::new();
        cpu.regs.pc = 0xC000;  // Start in WRAM for testing
        let bus = Bus::new();
        (cpu, bus)
    }

    #[test]
    fn test_nop() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0x00);  // NOP

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.regs.pc, 0xC001);
    }

    #[test]
    fn test_ld_b_n() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0x06);  // LD B, n
        bus.write(0xC001, 0x42);  // n = 0x42

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.regs.b, 0x42);
        assert_eq!(cpu.regs.pc, 0xC002);
    }

    #[test]
    fn test_ld_bc_nn() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0x01);  // LD BC, nn
        bus.write(0xC001, 0x34);  // low byte
        bus.write(0xC002, 0x12);  // high byte

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.regs.bc(), 0x1234);
    }

    #[test]
    fn test_xor_a() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.a = 0xFF;
        bus.write(0xC000, 0xAF);  // XOR A

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.a, 0x00);
        assert!(cpu.regs.f.z);
    }

    #[test]
    fn test_inc_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x0F;
        bus.write(0xC000, 0x04);  // INC B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x10);
        assert!(!cpu.regs.f.z);
        assert!(!cpu.regs.f.n);
        assert!(cpu.regs.f.h);  // Half carry from 0x0F to 0x10
    }

    #[test]
    fn test_dec_b() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.b = 0x10;
        bus.write(0xC000, 0x05);  // DEC B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.b, 0x0F);
        assert!(!cpu.regs.f.z);
        assert!(cpu.regs.f.n);
        assert!(cpu.regs.f.h);  // Half borrow from 0x10 to 0x0F
    }

    #[test]
    fn test_jp_nn() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0xC3);  // JP nn
        bus.write(0xC001, 0x50);
        bus.write(0xC002, 0x01);  // 0x0150

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.pc, 0x0150);
    }

    #[test]
    fn test_jr_n() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0x18);  // JR n
        bus.write(0xC001, 0x10);  // offset +16

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.pc, 0xC012);  // 0xC002 + 0x10
    }

    #[test]
    fn test_jr_negative() {
        let (mut cpu, mut bus) = setup();
        bus.write(0xC000, 0x18);  // JR n
        bus.write(0xC001, 0xFE);  // offset -2

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.pc, 0xC000);  // 0xC002 + (-2) = 0xC000
    }

    #[test]
    fn test_push_pop() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.sp = 0xFFFE;
        cpu.regs.set_bc(0x1234);

        // PUSH BC
        bus.write(0xC000, 0xC5);
        cpu.step(&mut bus);
        assert_eq!(cpu.regs.sp, 0xFFFC);

        // POP DE
        bus.write(0xC001, 0xD1);
        cpu.step(&mut bus);
        assert_eq!(cpu.regs.de(), 0x1234);
        assert_eq!(cpu.regs.sp, 0xFFFE);
    }

    #[test]
    fn test_call_ret() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.sp = 0xFFFE;

        // CALL 0xC100 (within WRAM)
        bus.write(0xC000, 0xCD);
        bus.write(0xC001, 0x00);
        bus.write(0xC002, 0xC1);  // 0xC100
        cpu.step(&mut bus);

        assert_eq!(cpu.regs.pc, 0xC100);
        assert_eq!(cpu.regs.sp, 0xFFFC);

        // RET (at 0xC100)
        bus.write(0xC100, 0xC9);
        cpu.step(&mut bus);

        assert_eq!(cpu.regs.pc, 0xC003);
        assert_eq!(cpu.regs.sp, 0xFFFE);
    }

    #[test]
    fn test_add_a() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.a = 0x3C;
        cpu.regs.b = 0x0F;
        bus.write(0xC000, 0x80);  // ADD A, B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.a, 0x4B);
        assert!(!cpu.regs.f.z);
        assert!(!cpu.regs.f.n);
        assert!(cpu.regs.f.h);  // Half carry
        assert!(!cpu.regs.f.c);
    }

    #[test]
    fn test_sub_a() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.a = 0x10;
        cpu.regs.b = 0x01;
        bus.write(0xC000, 0x90);  // SUB B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.a, 0x0F);
        assert!(!cpu.regs.f.z);
        assert!(cpu.regs.f.n);
        assert!(cpu.regs.f.h);  // Half borrow
        assert!(!cpu.regs.f.c);
    }

    #[test]
    fn test_cp() {
        let (mut cpu, mut bus) = setup();
        cpu.regs.a = 0x10;
        cpu.regs.b = 0x10;
        bus.write(0xC000, 0xB8);  // CP B

        cpu.step(&mut bus);

        assert_eq!(cpu.regs.a, 0x10);  // A unchanged
        assert!(cpu.regs.f.z);  // A == B
        assert!(cpu.regs.f.n);
    }
}
