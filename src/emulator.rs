// Game Boy Emulator Core
//
// This module ties together all components (CPU, Bus, etc.) and
// provides the main emulation loop.

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

/// The main emulator structure
pub struct Emulator {
    pub cpu: Cpu,
    pub bus: Bus,
    /// Total cycles executed
    pub cycles: u64,
}

impl Emulator {
    /// Create a new emulator with a loaded cartridge
    pub fn new(cartridge: &Cartridge) -> Self {
        let mut bus = Bus::new();
        bus.load_rom(&cartridge.rom);

        Self {
            cpu: Cpu::new(),
            bus,
            cycles: 0,
        }
    }

    /// Create a new emulator with raw ROM data
    pub fn with_rom(rom: &[u8]) -> Self {
        let mut bus = Bus::new();
        bus.load_rom(rom);

        Self {
            cpu: Cpu::new(),
            bus,
            cycles: 0,
        }
    }

    /// Execute one CPU instruction
    pub fn step(&mut self) -> u32 {
        let cycles = self.cpu.step(&mut self.bus);
        // Update timer and other hardware
        self.bus.tick(cycles);
        self.cycles += cycles as u64;
        cycles
    }

    /// Run until the CPU halts or reaches max cycles
    pub fn run_until_halt(&mut self, max_cycles: u64) -> bool {
        while !self.cpu.halted && self.cycles < max_cycles {
            self.step();
        }
        self.cpu.halted
    }

    /// Run for a specific number of cycles
    pub fn run_cycles(&mut self, cycles: u64) {
        let target = self.cycles + cycles;
        while self.cycles < target && !self.cpu.halted {
            self.step();
        }
    }

    /// Run until serial output contains a specific string or max cycles reached
    pub fn run_until_serial_contains(&mut self, needle: &str, max_cycles: u64) -> bool {
        while self.cycles < max_cycles && !self.cpu.halted {
            self.step();
            if self.bus.get_serial_output().contains(needle) {
                return true;
            }
        }
        false
    }

    /// Get current serial output
    pub fn get_serial_output(&self) -> String {
        self.bus.get_serial_output()
    }

    /// Check if test passed (output contains "Passed")
    pub fn test_passed(&self) -> bool {
        let output = self.get_serial_output();
        output.contains("Passed") || output.contains("passed")
    }

    /// Check if test failed (output contains "Failed")
    pub fn test_failed(&self) -> bool {
        let output = self.get_serial_output();
        output.contains("Failed") || output.contains("failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_creation() {
        // Create a minimal ROM
        let rom = vec![0u8; 0x8000];
        let emu = Emulator::with_rom(&rom);

        assert_eq!(emu.cpu.regs.pc, 0x0100);
        assert_eq!(emu.cycles, 0);
    }

    #[test]
    fn test_serial_output() {
        // Create a ROM that outputs "Hi" via serial
        let mut rom = vec![0u8; 0x8000];

        // Program at 0x0100:
        // LD A, 'H'
        // LD (0xFF01), A
        // LD A, 0x81
        // LD (0xFF02), A
        // LD A, 'i'
        // LD (0xFF01), A
        // LD A, 0x81
        // LD (0xFF02), A
        // HALT
        let program: &[u8] = &[
            0x3E, b'H',       // LD A, 'H'
            0xE0, 0x01,       // LDH (0x01), A  -> (0xFF01)
            0x3E, 0x81,       // LD A, 0x81
            0xE0, 0x02,       // LDH (0x02), A  -> (0xFF02)
            0x3E, b'i',       // LD A, 'i'
            0xE0, 0x01,       // LDH (0x01), A
            0x3E, 0x81,       // LD A, 0x81
            0xE0, 0x02,       // LDH (0x02), A
            0x76,             // HALT
        ];

        for (i, byte) in program.iter().enumerate() {
            rom[0x0100 + i] = *byte;
        }

        let mut emu = Emulator::with_rom(&rom);
        emu.run_until_halt(10000);

        assert_eq!(emu.get_serial_output(), "Hi");
    }

    #[test]
    fn test_run_cycles() {
        let rom = vec![0u8; 0x8000]; // All NOPs
        let mut emu = Emulator::with_rom(&rom);

        emu.run_cycles(100);

        // Each NOP is 4 cycles, so we should have executed ~25 NOPs
        assert!(emu.cycles >= 100);
    }
}
