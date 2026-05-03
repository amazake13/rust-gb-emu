// Memory Bank Controllers (MBC)
//
// Game Boy cartridges use MBCs to access more than 32KB of ROM
// and to provide external RAM (often battery-backed for saves).
//
// Common MBC types:
//   - ROM Only: No banking, max 32KB ROM
//   - MBC1: Up to 2MB ROM, 32KB RAM
//   - MBC2: Up to 256KB ROM, 512x4 bits RAM
//   - MBC3: Up to 2MB ROM, 32KB RAM, RTC
//   - MBC5: Up to 8MB ROM, 128KB RAM

mod mbc1;
mod mbc3;
mod no_mbc;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use no_mbc::NoMbc;

/// Trait for Memory Bank Controllers
pub trait Mbc {
    /// Read a byte from the cartridge
    fn read(&self, addr: u16) -> u8;

    /// Write a byte to the cartridge (for MBC registers or RAM)
    fn write(&mut self, addr: u16, value: u8);

    /// Check if external RAM is enabled
    fn ram_enabled(&self) -> bool;

    /// Get the current ROM bank number (for debugging)
    fn current_rom_bank(&self) -> usize;

    /// Get the current RAM bank number (for debugging)
    fn current_ram_bank(&self) -> usize;
}

/// Create an MBC based on cartridge type
pub fn create_mbc(cartridge_type: u8, rom: Vec<u8>, ram_size: usize) -> Box<dyn Mbc> {
    match cartridge_type {
        // ROM Only
        0x00 => Box::new(NoMbc::new(rom)),

        // MBC1
        0x01 => Box::new(Mbc1::new(rom, 0)),           // MBC1
        0x02 => Box::new(Mbc1::new(rom, ram_size)),    // MBC1+RAM
        0x03 => Box::new(Mbc1::new(rom, ram_size)),    // MBC1+RAM+BATTERY

        // MBC2
        0x05 | 0x06 => {
            // MBC2 has built-in 512x4 bits RAM
            Box::new(Mbc1::new(rom, 512)) // Use MBC1 as placeholder
        }

        // MBC3
        0x0F => Box::new(Mbc3::new(rom, 0)),           // MBC3+TIMER+BATTERY
        0x10 => Box::new(Mbc3::new(rom, ram_size)),    // MBC3+TIMER+RAM+BATTERY
        0x11 => Box::new(Mbc3::new(rom, 0)),           // MBC3
        0x12 => Box::new(Mbc3::new(rom, ram_size)),    // MBC3+RAM
        0x13 => Box::new(Mbc3::new(rom, ram_size)),    // MBC3+RAM+BATTERY

        // MBC5
        0x19..=0x1E => {
            // MBC5 - use MBC1 as placeholder for now
            Box::new(Mbc1::new(rom, ram_size))
        }

        // Unknown or unsupported - fall back to ROM only
        _ => {
            eprintln!("Warning: Unsupported MBC type 0x{:02X}, using ROM only", cartridge_type);
            Box::new(NoMbc::new(rom))
        }
    }
}
