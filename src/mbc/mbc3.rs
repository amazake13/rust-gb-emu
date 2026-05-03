// MBC3 (Memory Bank Controller 3)
//
// Features:
//   - Up to 2MB ROM (128 banks of 16KB)
//   - Up to 32KB RAM (4 banks of 8KB)
//   - Real-Time Clock (RTC) with 5 registers
//
// Memory Map:
//   0x0000-0x3FFF: ROM Bank 00 (fixed)
//   0x4000-0x7FFF: ROM Bank 01-7F (switchable)
//   0xA000-0xBFFF: RAM Bank 00-03 or RTC Registers
//
// Registers:
//   0x0000-0x1FFF: RAM/RTC Enable (write 0x0A to enable)
//   0x2000-0x3FFF: ROM Bank Number (7 bits, 0x01-0x7F)
//   0x4000-0x5FFF: RAM Bank Number (0x00-0x03) or RTC Register Select (0x08-0x0C)
//   0x6000-0x7FFF: Latch Clock Data (write 0x00 then 0x01 to latch)

use super::Mbc;

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,      // Also used for RTC register select
    rtc_latched: bool,
    latch_prepare: bool,
    // RTC registers (not fully implemented)
    rtc_s: u8,
    rtc_m: u8,
    rtc_h: u8,
    rtc_dl: u8,
    rtc_dh: u8,
    rom_bank_count: usize,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        let rom_bank_count = (rom.len() / 0x4000).max(2);
        Self {
            rom,
            ram: vec![0; ram_size.max(0x2000)],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc_latched: false,
            latch_prepare: false,
            rtc_s: 0,
            rtc_m: 0,
            rtc_h: 0,
            rtc_dl: 0,
            rtc_dh: 0,
            rom_bank_count,
        }
    }

    fn effective_rom_bank(&self) -> usize {
        let bank = if self.rom_bank == 0 { 1 } else { self.rom_bank as usize };
        bank % self.rom_bank_count
    }

    fn read_rtc(&self) -> u8 {
        match self.ram_bank {
            0x08 => self.rtc_s,
            0x09 => self.rtc_m,
            0x0A => self.rtc_h,
            0x0B => self.rtc_dl,
            0x0C => self.rtc_dh,
            _ => 0xFF,
        }
    }

    fn write_rtc(&mut self, value: u8) {
        match self.ram_bank {
            0x08 => self.rtc_s = value & 0x3F,
            0x09 => self.rtc_m = value & 0x3F,
            0x0A => self.rtc_h = value & 0x1F,
            0x0B => self.rtc_dl = value,
            0x0C => self.rtc_dh = value & 0xC1,
            _ => {}
        }
    }
}

impl Mbc for Mbc3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => {
                if (addr as usize) < self.rom.len() {
                    self.rom[addr as usize]
                } else {
                    0xFF
                }
            }

            // ROM Bank X (0x4000-0x7FFF)
            0x4000..=0x7FFF => {
                let bank = self.effective_rom_bank();
                let offset = bank * 0x4000 + ((addr - 0x4000) as usize);
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0xFF
                }
            }

            // External RAM or RTC (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                if self.ram_bank <= 0x03 {
                    // RAM access
                    let bank = self.ram_bank as usize;
                    let offset = bank * 0x2000 + ((addr - 0xA000) as usize);
                    if offset < self.ram.len() {
                        self.ram[offset]
                    } else {
                        0xFF
                    }
                } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                    // RTC register access
                    self.read_rtc()
                } else {
                    0xFF
                }
            }

            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM/RTC Enable (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM Bank Number (0x2000-0x3FFF)
            0x2000..=0x3FFF => {
                self.rom_bank = value & 0x7F;
            }

            // RAM Bank / RTC Select (0x4000-0x5FFF)
            0x4000..=0x5FFF => {
                self.ram_bank = value;
            }

            // Latch Clock Data (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                if !self.latch_prepare && value == 0x00 {
                    self.latch_prepare = true;
                } else if self.latch_prepare && value == 0x01 {
                    // Latch current time (not implemented - would copy current time to latched)
                    self.rtc_latched = true;
                    self.latch_prepare = false;
                } else {
                    self.latch_prepare = false;
                }
            }

            // External RAM or RTC (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }

                if self.ram_bank <= 0x03 {
                    // RAM access
                    let bank = self.ram_bank as usize;
                    let offset = bank * 0x2000 + ((addr - 0xA000) as usize);
                    if offset < self.ram.len() {
                        self.ram[offset] = value;
                    }
                } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                    // RTC register write
                    self.write_rtc(value);
                }
            }

            _ => {}
        }
    }

    fn ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    fn current_rom_bank(&self) -> usize {
        self.effective_rom_bank()
    }

    fn current_ram_bank(&self) -> usize {
        if self.ram_bank <= 0x03 {
            self.ram_bank as usize
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom(banks: usize) -> Vec<u8> {
        let mut rom = vec![0; banks * 0x4000];
        for bank in 0..banks {
            let offset = bank * 0x4000;
            rom[offset] = bank as u8;
        }
        rom
    }

    #[test]
    fn test_rom_banking() {
        let rom = create_test_rom(8);
        let mut mbc = Mbc3::new(rom, 0x8000);

        // Bank 0 fixed
        assert_eq!(mbc.read(0x0000), 0);

        // Default bank 1
        assert_eq!(mbc.read(0x4000), 1);

        // Switch to bank 5
        mbc.write(0x2000, 5);
        assert_eq!(mbc.read(0x4000), 5);
    }

    #[test]
    fn test_ram_banking() {
        let rom = create_test_rom(2);
        let mut mbc = Mbc3::new(rom, 0x8000);

        // Enable RAM
        mbc.write(0x0000, 0x0A);

        // Write to bank 0
        mbc.write(0x4000, 0);
        mbc.write(0xA000, 0x11);

        // Write to bank 1
        mbc.write(0x4000, 1);
        mbc.write(0xA000, 0x22);

        // Read back
        mbc.write(0x4000, 0);
        assert_eq!(mbc.read(0xA000), 0x11);

        mbc.write(0x4000, 1);
        assert_eq!(mbc.read(0xA000), 0x22);
    }
}
