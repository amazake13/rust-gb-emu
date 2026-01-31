// Cartridge Module
//
// Game Boy cartridges contain ROM data and optionally RAM.
// The cartridge header (0x0100-0x014F) contains important metadata:
//
// 0x0100-0x0103: Entry point (usually NOP + JP)
// 0x0104-0x0133: Nintendo logo (must match for boot)
// 0x0134-0x0143: Title (uppercase ASCII)
// 0x0143: CGB flag
// 0x0144-0x0145: New licensee code
// 0x0146: SGB flag
// 0x0147: Cartridge type (MBC type)
// 0x0148: ROM size
// 0x0149: RAM size
// 0x014A: Destination code
// 0x014B: Old licensee code
// 0x014C: Mask ROM version
// 0x014D: Header checksum
// 0x014E-0x014F: Global checksum

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Cartridge types (MBC - Memory Bank Controller)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Unknown(u8),
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x0F => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1A => CartridgeType::Mbc5Ram,
            0x1B => CartridgeType::Mbc5RamBattery,
            _ => CartridgeType::Unknown(value),
        }
    }
}

/// Cartridge information parsed from header
#[derive(Debug)]
pub struct CartridgeInfo {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: usize,
    pub header_checksum: u8,
    pub checksum_valid: bool,
}

/// Cartridge data and metadata
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub info: CartridgeInfo,
}

impl Cartridge {
    /// Load a ROM file from disk
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(&path)
            .map_err(|e| format!("Failed to open ROM file: {}", e))?;

        let mut rom = Vec::new();
        file.read_to_end(&mut rom)
            .map_err(|e| format!("Failed to read ROM file: {}", e))?;

        Self::from_bytes(rom)
    }

    /// Load ROM from bytes
    pub fn from_bytes(rom: Vec<u8>) -> Result<Self, String> {
        if rom.len() < 0x150 {
            return Err("ROM too small (must be at least 336 bytes for header)".to_string());
        }

        let info = Self::parse_header(&rom)?;
        Ok(Self { rom, info })
    }

    /// Parse cartridge header
    fn parse_header(rom: &[u8]) -> Result<CartridgeInfo, String> {
        // Extract title (0x0134-0x0143)
        let title_bytes = &rom[0x0134..=0x0143];
        let title = title_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();

        // Cartridge type (0x0147)
        let cartridge_type = CartridgeType::from(rom[0x0147]);

        // ROM size (0x0148): 32KB << value
        let rom_size = match rom[0x0148] {
            0x00 => 32 * 1024,      // 32KB (no banking)
            0x01 => 64 * 1024,      // 64KB (4 banks)
            0x02 => 128 * 1024,     // 128KB (8 banks)
            0x03 => 256 * 1024,     // 256KB (16 banks)
            0x04 => 512 * 1024,     // 512KB (32 banks)
            0x05 => 1024 * 1024,    // 1MB (64 banks)
            0x06 => 2048 * 1024,    // 2MB (128 banks)
            0x07 => 4096 * 1024,    // 4MB (256 banks)
            0x08 => 8192 * 1024,    // 8MB (512 banks)
            _ => 32 * 1024,         // Default
        };

        // RAM size (0x0149)
        let ram_size = match rom[0x0149] {
            0x00 => 0,
            0x01 => 2 * 1024,       // 2KB (unused)
            0x02 => 8 * 1024,       // 8KB
            0x03 => 32 * 1024,      // 32KB (4 banks)
            0x04 => 128 * 1024,     // 128KB (16 banks)
            0x05 => 64 * 1024,      // 64KB (8 banks)
            _ => 0,
        };

        // Header checksum (0x014D)
        let header_checksum = rom[0x014D];

        // Verify header checksum
        // x = 0
        // for i in 0x0134..=0x014C: x = x - rom[i] - 1
        let mut checksum: u8 = 0;
        for i in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(rom[i]).wrapping_sub(1);
        }
        let checksum_valid = checksum == header_checksum;

        Ok(CartridgeInfo {
            title,
            cartridge_type,
            rom_size,
            ram_size,
            header_checksum,
            checksum_valid,
        })
    }

    /// Read a byte from ROM
    pub fn read(&self, addr: u16) -> u8 {
        if (addr as usize) < self.rom.len() {
            self.rom[addr as usize]
        } else {
            0xFF
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_rom() -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000]; // 32KB

        // Entry point at 0x0100
        rom[0x0100] = 0x00; // NOP
        rom[0x0101] = 0xC3; // JP
        rom[0x0102] = 0x50; // addr low
        rom[0x0103] = 0x01; // addr high (0x0150)

        // Title at 0x0134
        let title = b"TEST";
        rom[0x0134..0x0134 + title.len()].copy_from_slice(title);

        // Cartridge type: ROM only
        rom[0x0147] = 0x00;

        // ROM size: 32KB
        rom[0x0148] = 0x00;

        // RAM size: None
        rom[0x0149] = 0x00;

        // Calculate header checksum
        let mut checksum: u8 = 0;
        for i in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(rom[i]).wrapping_sub(1);
        }
        rom[0x014D] = checksum;

        rom
    }

    #[test]
    fn test_load_rom() {
        let rom = create_minimal_rom();
        let cart = Cartridge::from_bytes(rom).unwrap();

        assert_eq!(cart.info.title, "TEST");
        assert_eq!(cart.info.cartridge_type, CartridgeType::RomOnly);
        assert_eq!(cart.info.rom_size, 32 * 1024);
        assert_eq!(cart.info.ram_size, 0);
        assert!(cart.info.checksum_valid);
    }

    #[test]
    fn test_cartridge_type_parsing() {
        assert_eq!(CartridgeType::from(0x00), CartridgeType::RomOnly);
        assert_eq!(CartridgeType::from(0x01), CartridgeType::Mbc1);
        assert_eq!(CartridgeType::from(0x03), CartridgeType::Mbc1RamBattery);
        assert_eq!(CartridgeType::from(0x13), CartridgeType::Mbc3RamBattery);
        assert_eq!(CartridgeType::from(0x1B), CartridgeType::Mbc5RamBattery);
    }

    #[test]
    fn test_rom_too_small() {
        let rom = vec![0u8; 100];
        let result = Cartridge::from_bytes(rom);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_rom() {
        let mut rom = create_minimal_rom();
        rom[0x0150] = 0xAB;
        rom[0x0151] = 0xCD;

        let cart = Cartridge::from_bytes(rom).unwrap();

        assert_eq!(cart.read(0x0150), 0xAB);
        assert_eq!(cart.read(0x0151), 0xCD);
    }
}
