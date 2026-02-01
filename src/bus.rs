// Memory Bus
//
// The Game Boy has a 16-bit address bus (64KB addressable space).
// Different address ranges map to different hardware components.
//
// Memory Map:
// 0x0000-0x3FFF: ROM Bank 0 (16KB) - Fixed cartridge ROM
// 0x4000-0x7FFF: ROM Bank N (16KB) - Switchable cartridge ROM
// 0x8000-0x9FFF: VRAM (8KB) - Video RAM for tiles and maps
// 0xA000-0xBFFF: External RAM (8KB) - Cartridge RAM (battery-backed for saves)
// 0xC000-0xDFFF: WRAM (8KB) - Work RAM
// 0xE000-0xFDFF: Echo RAM - Mirror of C000-DDFF (not recommended to use)
// 0xFE00-0xFE9F: OAM (160B) - Object Attribute Memory (sprite data)
// 0xFEA0-0xFEFF: Unusable - Returns 0xFF on read
// 0xFF00-0xFF7F: I/O Registers - Hardware control registers
// 0xFF80-0xFFFE: HRAM (127B) - High RAM (fast access)
// 0xFFFF: IE Register - Interrupt Enable register

use crate::timer::Timer;

/// Memory Bus - handles all memory read/write operations
pub struct Bus {
    /// Cartridge ROM (32KB for now, will expand with MBC support)
    rom: Vec<u8>,
    /// Video RAM (8KB)
    vram: [u8; 0x2000],
    /// External RAM (8KB, cartridge RAM)
    external_ram: [u8; 0x2000],
    /// Work RAM (8KB)
    wram: [u8; 0x2000],
    /// High RAM (127 bytes)
    hram: [u8; 0x7F],
    /// I/O Registers (128 bytes, 0xFF00-0xFF7F)
    io: [u8; 0x80],
    /// OAM - Object Attribute Memory (160 bytes)
    oam: [u8; 0xA0],
    /// Interrupt Enable register (0xFFFF)
    ie: u8,
    /// Serial output buffer (for test ROMs)
    pub serial_output: Vec<u8>,
    /// Timer
    pub timer: Timer,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: vec![0; 0x8000], // 32KB ROM space
            vram: [0; 0x2000],
            external_ram: [0; 0x2000],
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            io: [0; 0x80],
            oam: [0; 0xA0],
            ie: 0,
            serial_output: Vec::new(),
            timer: Timer::new(),
        }
    }

    /// Get serial output as string
    pub fn get_serial_output(&self) -> String {
        String::from_utf8_lossy(&self.serial_output).to_string()
    }

    /// Update timer and check for interrupts
    pub fn tick(&mut self, cycles: u32) {
        self.timer.tick(cycles);

        // Check for timer interrupt
        if self.timer.take_interrupt() {
            // Set Timer interrupt flag (bit 2 of IF)
            self.io[0x0F] |= 0x04;
        }
    }

    /// Load ROM data into memory
    pub fn load_rom(&mut self, data: &[u8]) {
        let len = data.len().min(self.rom.len());
        self.rom[..len].copy_from_slice(&data[..len]);
    }

    /// Read a byte from the given address
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0 (fixed)
            0x0000..=0x3FFF => self.rom[addr as usize],

            // ROM Bank N (switchable) - for now just read from ROM
            0x4000..=0x7FFF => self.rom[addr as usize],

            // Video RAM
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],

            // External RAM (cartridge)
            0xA000..=0xBFFF => self.external_ram[(addr - 0xA000) as usize],

            // Work RAM
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],

            // Echo RAM (mirror of C000-DDFF)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],

            // OAM (Object Attribute Memory)
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],

            // Unusable area
            0xFEA0..=0xFEFF => 0xFF,

            // I/O Registers
            0xFF00..=0xFF7F => self.read_io(addr),

            // High RAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // Interrupt Enable register
            0xFFFF => self.ie,
        }
    }

    /// Write a byte to the given address
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // ROM is read-only (writes go to MBC, will implement later)
            0x0000..=0x7FFF => {
                // MBC control - ignore for now
            }

            // Video RAM
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,

            // External RAM (cartridge)
            0xA000..=0xBFFF => self.external_ram[(addr - 0xA000) as usize] = value,

            // Work RAM
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,

            // Echo RAM (writes also go to WRAM)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,

            // OAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,

            // Unusable area - writes ignored
            0xFEA0..=0xFEFF => {}

            // I/O Registers
            0xFF00..=0xFF7F => self.write_io(addr, value),

            // High RAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,

            // Interrupt Enable register
            0xFFFF => self.ie = value,
        }
    }

    /// Read from I/O registers
    fn read_io(&self, addr: u16) -> u8 {
        let offset = (addr - 0xFF00) as usize;
        match addr {
            // Joypad - will implement later, return 0xFF for now
            0xFF00 => 0xFF,

            // Serial transfer - stub
            0xFF01..=0xFF02 => self.io[offset],

            // Timer registers
            0xFF04 => self.timer.div(),           // DIV
            0xFF05 => self.timer.tima,            // TIMA
            0xFF06 => self.timer.tma,             // TMA
            0xFF07 => self.timer.tac | 0xF8,      // TAC (upper bits return 1)

            // Interrupt Flag (IF)
            0xFF0F => self.io[offset] | 0xE0,     // Upper bits always 1

            // Sound registers - stub for now
            0xFF10..=0xFF3F => self.io[offset],

            // LCD registers
            0xFF40..=0xFF4B => self.io[offset],

            // Other I/O
            _ => self.io[offset],
        }
    }

    /// Write to I/O registers
    fn write_io(&mut self, addr: u16, value: u8) {
        let offset = (addr - 0xFF00) as usize;
        match addr {
            // Serial Control (SC) - 0xFF02
            // When bit 7 is set (0x81), a transfer is initiated
            // For test ROMs, we capture the data byte (SB at 0xFF01)
            0xFF02 => {
                self.io[offset] = value;
                if value == 0x81 {
                    // Transfer requested - capture the byte from SB
                    let sb = self.io[0x01]; // 0xFF01 - SB register
                    self.serial_output.push(sb);
                }
            }

            // Timer registers
            0xFF04 => self.timer.reset_div(),     // DIV - any write resets
            0xFF05 => self.timer.tima = value,    // TIMA
            0xFF06 => self.timer.tma = value,     // TMA
            0xFF07 => self.timer.write_tac(value), // TAC

            // Interrupt Flag (IF)
            0xFF0F => self.io[offset] = value & 0x1F,  // Only lower 5 bits

            // Normal I/O write
            _ => self.io[offset] = value,
        }
    }

    /// Read a 16-bit value (little-endian)
    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a 16-bit value (little-endian)
    pub fn write16(&mut self, addr: u16, value: u16) {
        self.write(addr, (value & 0xFF) as u8);
        self.write(addr.wrapping_add(1), (value >> 8) as u8);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wram_read_write() {
        let mut bus = Bus::new();

        // Write to WRAM
        bus.write(0xC000, 0x42);
        bus.write(0xDFFF, 0x69);

        // Read back
        assert_eq!(bus.read(0xC000), 0x42);
        assert_eq!(bus.read(0xDFFF), 0x69);
    }

    #[test]
    fn test_echo_ram() {
        let mut bus = Bus::new();

        // Write to WRAM
        bus.write(0xC000, 0xAB);

        // Read from Echo RAM (should mirror WRAM)
        assert_eq!(bus.read(0xE000), 0xAB);

        // Write to Echo RAM
        bus.write(0xE100, 0xCD);

        // Should be reflected in WRAM
        assert_eq!(bus.read(0xC100), 0xCD);
    }

    #[test]
    fn test_hram() {
        let mut bus = Bus::new();

        bus.write(0xFF80, 0x11);
        bus.write(0xFFFE, 0x22);

        assert_eq!(bus.read(0xFF80), 0x11);
        assert_eq!(bus.read(0xFFFE), 0x22);
    }

    #[test]
    fn test_vram() {
        let mut bus = Bus::new();

        bus.write(0x8000, 0xAA);
        bus.write(0x9FFF, 0xBB);

        assert_eq!(bus.read(0x8000), 0xAA);
        assert_eq!(bus.read(0x9FFF), 0xBB);
    }

    #[test]
    fn test_ie_register() {
        let mut bus = Bus::new();

        bus.write(0xFFFF, 0x1F);
        assert_eq!(bus.read(0xFFFF), 0x1F);
    }

    #[test]
    fn test_unusable_area() {
        let bus = Bus::new();

        // Unusable area should return 0xFF
        assert_eq!(bus.read(0xFEA0), 0xFF);
        assert_eq!(bus.read(0xFEFF), 0xFF);
    }

    #[test]
    fn test_16bit_read_write() {
        let mut bus = Bus::new();

        // Write 16-bit value (little-endian)
        bus.write16(0xC000, 0x1234);

        // Low byte at lower address, high byte at higher address
        assert_eq!(bus.read(0xC000), 0x34); // Low byte
        assert_eq!(bus.read(0xC001), 0x12); // High byte

        // Read back as 16-bit
        assert_eq!(bus.read16(0xC000), 0x1234);
    }

    #[test]
    fn test_rom_read() {
        let mut bus = Bus::new();

        // Load some ROM data
        let rom_data = vec![0x00, 0x01, 0x02, 0x03];
        bus.load_rom(&rom_data);

        assert_eq!(bus.read(0x0000), 0x00);
        assert_eq!(bus.read(0x0001), 0x01);
        assert_eq!(bus.read(0x0002), 0x02);
        assert_eq!(bus.read(0x0003), 0x03);
    }

    #[test]
    fn test_div_reset() {
        let mut bus = Bus::new();

        // Set DIV to some value directly (simulating timer tick)
        bus.io[0x04] = 0xAB;

        // Writing any value should reset to 0
        bus.write(0xFF04, 0x42);
        assert_eq!(bus.read(0xFF04), 0x00);
    }
}
