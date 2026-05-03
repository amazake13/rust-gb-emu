// PPU Registers
//
// LCDC (0xFF40) - LCD Control
// STAT (0xFF41) - LCD Status

/// LCD Control Register (0xFF40)
/// Bit 7: LCD Enable (0=Off, 1=On)
/// Bit 6: Window Tile Map (0=9800-9BFF, 1=9C00-9FFF)
/// Bit 5: Window Enable (0=Off, 1=On)
/// Bit 4: BG/Window Tile Data (0=8800-97FF, 1=8000-8FFF)
/// Bit 3: BG Tile Map (0=9800-9BFF, 1=9C00-9FFF)
/// Bit 2: OBJ Size (0=8x8, 1=8x16)
/// Bit 1: OBJ Enable (0=Off, 1=On)
/// Bit 0: BG/Window Enable (0=Off, 1=On)
#[derive(Debug, Clone, Copy)]
pub struct LcdControl(pub u8);

impl LcdControl {
    /// Bit 7: LCD Display Enable
    pub fn lcd_enable(&self) -> bool {
        self.0 & 0x80 != 0
    }

    /// Bit 6: Window Tile Map Area (0=9800-9BFF, 1=9C00-9FFF)
    pub fn window_tile_map(&self) -> bool {
        self.0 & 0x40 != 0
    }

    /// Bit 5: Window Display Enable
    pub fn window_enable(&self) -> bool {
        self.0 & 0x20 != 0
    }

    /// Bit 4: BG & Window Tile Data Area (0=8800-97FF signed, 1=8000-8FFF unsigned)
    pub fn bg_window_tile_data(&self) -> bool {
        self.0 & 0x10 != 0
    }

    /// Bit 3: BG Tile Map Area (0=9800-9BFF, 1=9C00-9FFF)
    pub fn bg_tile_map(&self) -> bool {
        self.0 & 0x08 != 0
    }

    /// Bit 2: OBJ Size (0=8x8, 1=8x16)
    pub fn obj_size(&self) -> bool {
        self.0 & 0x04 != 0
    }

    /// Bit 1: OBJ Enable
    pub fn obj_enable(&self) -> bool {
        self.0 & 0x02 != 0
    }

    /// Bit 0: BG/Window Enable (on DMG, 0=both off, 1=on)
    pub fn bg_enable(&self) -> bool {
        self.0 & 0x01 != 0
    }
}

/// LCD Status Register (0xFF41)
/// Bit 6: LYC=LY Interrupt Enable
/// Bit 5: Mode 2 OAM Interrupt Enable
/// Bit 4: Mode 1 VBlank Interrupt Enable
/// Bit 3: Mode 0 HBlank Interrupt Enable
/// Bit 2: LYC=LY Coincidence Flag (read-only)
/// Bit 1-0: Mode Flag (read-only)
#[derive(Debug, Clone, Copy)]
pub struct LcdStatus(pub u8);

impl LcdStatus {
    /// Bit 6: LYC=LY Interrupt Enable
    pub fn lyc_interrupt(&self) -> bool {
        self.0 & 0x40 != 0
    }

    /// Bit 5: Mode 2 (OAM) Interrupt Enable
    pub fn oam_interrupt(&self) -> bool {
        self.0 & 0x20 != 0
    }

    /// Bit 4: Mode 1 (VBlank) Interrupt Enable
    pub fn vblank_interrupt(&self) -> bool {
        self.0 & 0x10 != 0
    }

    /// Bit 3: Mode 0 (HBlank) Interrupt Enable
    pub fn hblank_interrupt(&self) -> bool {
        self.0 & 0x08 != 0
    }

    /// Bit 2: LYC=LY Coincidence Flag
    pub fn coincidence(&self) -> bool {
        self.0 & 0x04 != 0
    }

    /// Set coincidence flag
    pub fn set_coincidence(&mut self, value: bool) {
        if value {
            self.0 |= 0x04;
        } else {
            self.0 &= !0x04;
        }
    }

    /// Bits 0-1: Current Mode
    pub fn mode(&self) -> u8 {
        self.0 & 0x03
    }

    /// Set current mode
    pub fn set_mode(&mut self, mode: u8) {
        self.0 = (self.0 & 0xFC) | (mode & 0x03);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcdc() {
        // After boot: 0x91 = 10010001
        let lcdc = LcdControl(0x91);
        assert!(lcdc.lcd_enable()); // Bit 7
        assert!(!lcdc.window_tile_map()); // Bit 6
        assert!(!lcdc.window_enable()); // Bit 5
        assert!(lcdc.bg_window_tile_data()); // Bit 4
        assert!(!lcdc.bg_tile_map()); // Bit 3
        assert!(!lcdc.obj_size()); // Bit 2
        assert!(!lcdc.obj_enable()); // Bit 1
        assert!(lcdc.bg_enable()); // Bit 0
    }

    #[test]
    fn test_stat() {
        let mut stat = LcdStatus(0x00);

        stat.set_mode(2);
        assert_eq!(stat.mode(), 2);

        stat.set_coincidence(true);
        assert!(stat.coincidence());
        assert_eq!(stat.0, 0x06); // Mode 2 + coincidence

        stat.set_coincidence(false);
        assert!(!stat.coincidence());
        assert_eq!(stat.0, 0x02);
    }
}
