// MBC1 (Memory Bank Controller 1)
//
// Features:
//   - Up to 2MB ROM (125 banks of 16KB, banks 0x20/0x40/0x60 map to 0x21/0x41/0x61)
//   - Up to 32KB RAM (4 banks of 8KB)
//   - Two modes: ROM banking mode (default) and RAM banking mode
//
// Memory Map:
//   0x0000-0x3FFF: ROM Bank 00 (fixed, or bank 0x20/0x40/0x60 in mode 1)
//   0x4000-0x7FFF: ROM Bank 01-7F (switchable)
//   0xA000-0xBFFF: RAM Bank 00-03 (if RAM enabled)
//
// Registers (directly in ROM space as writes):
//   0x0000-0x1FFF: RAM Enable (write 0x0A to enable)
//   0x2000-0x3FFF: ROM Bank Number (lower 5 bits)
//   0x4000-0x5FFF: RAM Bank Number OR upper ROM bank bits
//   0x6000-0x7FFF: Banking Mode Select (0=ROM, 1=RAM)

use super::Mbc;

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,      // Lower 5 bits of ROM bank
    ram_bank: u8,      // RAM bank OR upper 2 bits of ROM bank
    banking_mode: bool, // false = ROM mode, true = RAM mode
    rom_bank_count: usize,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        let rom_bank_count = (rom.len() / 0x4000).max(2);
        Self {
            rom,
            ram: vec![0; ram_size.max(0x2000)], // At least 8KB for simplicity
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: false,
            rom_bank_count,
        }
    }

    /// Get the effective ROM bank for 0x0000-0x3FFF region
    fn rom_bank_0(&self) -> usize {
        if self.banking_mode {
            // In RAM banking mode, upper bits affect bank 0 region too
            ((self.ram_bank as usize) << 5) % self.rom_bank_count
        } else {
            0
        }
    }

    /// Get the effective ROM bank for 0x4000-0x7FFF region
    fn rom_bank_x(&self) -> usize {
        let mut bank = self.rom_bank as usize;

        // Bank 0 is not allowed, maps to bank 1
        if bank == 0 {
            bank = 1;
        }

        // Add upper 2 bits from ram_bank
        bank |= (self.ram_bank as usize) << 5;

        // Mask to available banks
        bank % self.rom_bank_count
    }

    /// Get the effective RAM bank
    fn effective_ram_bank(&self) -> usize {
        if self.banking_mode {
            (self.ram_bank as usize) & 0x03
        } else {
            0
        }
    }
}

impl Mbc for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => {
                let bank = self.rom_bank_0();
                let offset = bank * 0x4000 + (addr as usize);
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0xFF
                }
            }

            // ROM Bank X (0x4000-0x7FFF)
            0x4000..=0x7FFF => {
                let bank = self.rom_bank_x();
                let offset = bank * 0x4000 + ((addr - 0x4000) as usize);
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0xFF
                }
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if self.ram_enabled && !self.ram.is_empty() {
                    let bank = self.effective_ram_bank();
                    let offset = bank * 0x2000 + ((addr - 0xA000) as usize);
                    if offset < self.ram.len() {
                        self.ram[offset]
                    } else {
                        0xFF
                    }
                } else {
                    0xFF
                }
            }

            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM Enable (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                // Writing 0x0A enables RAM, anything else disables it
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM Bank Number (0x2000-0x3FFF)
            0x2000..=0x3FFF => {
                // Lower 5 bits select ROM bank
                self.rom_bank = value & 0x1F;
            }

            // RAM Bank Number / Upper ROM Bank (0x4000-0x5FFF)
            0x4000..=0x5FFF => {
                // 2 bits for RAM bank or upper ROM bank
                self.ram_bank = value & 0x03;
            }

            // Banking Mode Select (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                self.banking_mode = (value & 0x01) != 0;
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if self.ram_enabled && !self.ram.is_empty() {
                    let bank = self.effective_ram_bank();
                    let offset = bank * 0x2000 + ((addr - 0xA000) as usize);
                    if offset < self.ram.len() {
                        self.ram[offset] = value;
                    }
                }
            }

            _ => {}
        }
    }

    fn ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn current_rom_bank(&self) -> usize {
        self.rom_bank_x()
    }

    fn current_ram_bank(&self) -> usize {
        self.effective_ram_bank()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom(banks: usize) -> Vec<u8> {
        let mut rom = vec![0; banks * 0x4000];
        // Mark each bank with its number
        for bank in 0..banks {
            let offset = bank * 0x4000;
            rom[offset] = bank as u8;
        }
        rom
    }

    #[test]
    fn test_initial_state() {
        let rom = create_test_rom(4);
        let mbc = Mbc1::new(rom, 0x8000);

        // Should start with ROM bank 1
        assert_eq!(mbc.current_rom_bank(), 1);
        assert!(!mbc.ram_enabled());
    }

    #[test]
    fn test_rom_banking() {
        let rom = create_test_rom(8);
        let mut mbc = Mbc1::new(rom, 0);

        // Bank 0 is fixed
        assert_eq!(mbc.read(0x0000), 0);

        // Default bank 1
        assert_eq!(mbc.read(0x4000), 1);

        // Switch to bank 3
        mbc.write(0x2000, 3);
        assert_eq!(mbc.read(0x4000), 3);

        // Switch to bank 7
        mbc.write(0x2000, 7);
        assert_eq!(mbc.read(0x4000), 7);
    }

    #[test]
    fn test_bank_0_maps_to_1() {
        let rom = create_test_rom(4);
        let mut mbc = Mbc1::new(rom, 0);

        // Writing 0 should map to bank 1
        mbc.write(0x2000, 0);
        assert_eq!(mbc.current_rom_bank(), 1);
    }

    #[test]
    fn test_ram_enable() {
        let mut rom = create_test_rom(2);
        let mut mbc = Mbc1::new(rom, 0x2000);

        assert!(!mbc.ram_enabled());

        // Enable RAM with 0x0A
        mbc.write(0x0000, 0x0A);
        assert!(mbc.ram_enabled());

        // Disable with anything else
        mbc.write(0x0000, 0x00);
        assert!(!mbc.ram_enabled());
    }

    #[test]
    fn test_ram_read_write() {
        let rom = create_test_rom(2);
        let mut mbc = Mbc1::new(rom, 0x2000);

        // Enable RAM
        mbc.write(0x0000, 0x0A);

        // Write and read
        mbc.write(0xA000, 0x42);
        assert_eq!(mbc.read(0xA000), 0x42);

        // Disable RAM
        mbc.write(0x0000, 0x00);
        assert_eq!(mbc.read(0xA000), 0xFF);
    }
}
