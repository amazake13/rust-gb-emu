// No MBC (ROM Only)
//
// Simple cartridges with no memory bank controller.
// Maximum 32KB ROM, no external RAM.

use super::Mbc;

pub struct NoMbc {
    rom: Vec<u8>,
}

impl NoMbc {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Mbc for NoMbc {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM (0x0000-0x7FFF)
            0x0000..=0x7FFF => {
                if (addr as usize) < self.rom.len() {
                    self.rom[addr as usize]
                } else {
                    0xFF
                }
            }
            // External RAM (not available)
            0xA000..=0xBFFF => 0xFF,
            _ => 0xFF,
        }
    }

    fn write(&mut self, _addr: u16, _value: u8) {
        // ROM only - writes are ignored
    }

    fn ram_enabled(&self) -> bool {
        false
    }

    fn current_rom_bank(&self) -> usize {
        1
    }

    fn current_ram_bank(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_read() {
        let rom = vec![0x00, 0x01, 0x02, 0x03];
        let mbc = NoMbc::new(rom);

        assert_eq!(mbc.read(0x0000), 0x00);
        assert_eq!(mbc.read(0x0001), 0x01);
        assert_eq!(mbc.read(0x0002), 0x02);
        assert_eq!(mbc.read(0x0003), 0x03);
    }

    #[test]
    fn test_out_of_bounds() {
        let rom = vec![0x00; 0x100];
        let mbc = NoMbc::new(rom);

        // Reading beyond ROM returns 0xFF
        assert_eq!(mbc.read(0x7FFF), 0xFF);
    }

    #[test]
    fn test_no_ram() {
        let rom = vec![0x00; 0x8000];
        let mbc = NoMbc::new(rom);

        assert!(!mbc.ram_enabled());
        assert_eq!(mbc.read(0xA000), 0xFF);
    }
}
