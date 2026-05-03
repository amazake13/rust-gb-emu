// PPU (Pixel Processing Unit) - Game Boy Graphics
//
// The Game Boy PPU renders:
//   - 160x144 pixel display
//   - 4 shades of gray (2-bit per pixel)
//   - Background layer (256x256 virtual, scrollable)
//   - Window layer (overlays background)
//   - Up to 40 sprites (OAM entries)
//
// PPU Timing (per frame):
//   - 154 scanlines total (144 visible + 10 VBlank)
//   - 456 dots per scanline
//   - ~70224 dots per frame (~59.7 fps)
//
// PPU Modes:
//   Mode 2 (OAM Scan): 80 dots - Searching OAM for sprites on current line
//   Mode 3 (Drawing): 168-291 dots - Transferring pixels to LCD
//   Mode 0 (HBlank): 85-208 dots - Horizontal blank
//   Mode 1 (VBlank): 4560 dots - Vertical blank (10 scanlines)

pub mod registers;

use registers::*;

/// Screen dimensions
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// Total scanlines including VBlank
pub const TOTAL_SCANLINES: u8 = 154;

/// Dots per scanline
pub const DOTS_PER_LINE: u32 = 456;

/// PPU modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuMode {
    HBlank = 0,  // Mode 0
    VBlank = 1,  // Mode 1
    OamScan = 2, // Mode 2
    Drawing = 3, // Mode 3
}

/// Sprite attributes from OAM
#[derive(Debug, Clone, Copy, Default)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
}

impl Sprite {
    /// Priority: 0 = Above BG, 1 = Behind BG colors 1-3
    pub fn priority(&self) -> bool {
        self.flags & 0x80 != 0
    }

    /// Y flip
    pub fn y_flip(&self) -> bool {
        self.flags & 0x40 != 0
    }

    /// X flip
    pub fn x_flip(&self) -> bool {
        self.flags & 0x20 != 0
    }

    /// Palette: 0 = OBP0, 1 = OBP1
    pub fn palette(&self) -> bool {
        self.flags & 0x10 != 0
    }
}

/// The PPU state
pub struct Ppu {
    /// LCD Control register (0xFF40)
    pub lcdc: LcdControl,
    /// LCD Status register (0xFF41)
    pub stat: LcdStatus,
    /// Scroll Y (0xFF42)
    pub scy: u8,
    /// Scroll X (0xFF43)
    pub scx: u8,
    /// LY - Current scanline (0xFF44)
    pub ly: u8,
    /// LY Compare (0xFF45)
    pub lyc: u8,
    /// Background Palette (0xFF47)
    pub bgp: u8,
    /// Object Palette 0 (0xFF48)
    pub obp0: u8,
    /// Object Palette 1 (0xFF49)
    pub obp1: u8,
    /// Window Y (0xFF4A)
    pub wy: u8,
    /// Window X (0xFF4B)
    pub wx: u8,

    /// Video RAM (8KB)
    pub vram: [u8; 0x2000],
    /// OAM - Object Attribute Memory (160 bytes for 40 sprites)
    pub oam: [u8; 160],

    /// Current dot within the scanline (0-455)
    dot: u32,
    /// Current PPU mode
    mode: PpuMode,

    /// Frame buffer (160x144 pixels, 2-bit color values 0-3)
    pub framebuffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    /// Internal window line counter
    window_line: u8,
    /// Whether window was triggered this frame
    window_triggered: bool,

    /// VBlank interrupt request flag
    pub vblank_interrupt: bool,
    /// STAT interrupt request flag
    pub stat_interrupt: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcdc: LcdControl(0x91), // LCD on, BG on after boot
            stat: LcdStatus(0x00),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            vram: [0; 0x2000],
            oam: [0; 160],
            dot: 0,
            mode: PpuMode::OamScan,
            framebuffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            window_line: 0,
            window_triggered: false,
            vblank_interrupt: false,
            stat_interrupt: false,
        }
    }

    /// Tick the PPU by the given number of CPU cycles (T-cycles)
    pub fn tick(&mut self, cycles: u32) {
        if !self.lcdc.lcd_enable() {
            return;
        }

        self.vblank_interrupt = false;
        self.stat_interrupt = false;

        for _ in 0..cycles {
            self.dot += 1;

            match self.mode {
                PpuMode::OamScan => {
                    // Mode 2: OAM scan takes 80 dots
                    if self.dot >= 80 {
                        self.set_mode(PpuMode::Drawing);
                    }
                }
                PpuMode::Drawing => {
                    // Mode 3: Drawing takes variable time, we use 172 dots
                    if self.dot >= 80 + 172 {
                        self.render_scanline();
                        self.set_mode(PpuMode::HBlank);
                    }
                }
                PpuMode::HBlank => {
                    // Mode 0: HBlank until end of scanline
                    if self.dot >= DOTS_PER_LINE {
                        self.dot = 0;
                        self.ly += 1;
                        self.check_lyc();

                        if self.ly >= SCREEN_HEIGHT as u8 {
                            self.set_mode(PpuMode::VBlank);
                            self.vblank_interrupt = true;
                            self.window_triggered = false;
                            self.window_line = 0;
                        } else {
                            self.set_mode(PpuMode::OamScan);
                        }
                    }
                }
                PpuMode::VBlank => {
                    // Mode 1: VBlank for 10 scanlines
                    if self.dot >= DOTS_PER_LINE {
                        self.dot = 0;
                        self.ly += 1;
                        self.check_lyc();

                        if self.ly >= TOTAL_SCANLINES {
                            self.ly = 0;
                            self.check_lyc();
                            self.set_mode(PpuMode::OamScan);
                        }
                    }
                }
            }
        }
    }

    /// Set PPU mode and potentially trigger STAT interrupt
    fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        self.stat.set_mode(mode as u8);

        // Check STAT interrupt conditions
        let trigger = match mode {
            PpuMode::HBlank => self.stat.hblank_interrupt(),
            PpuMode::VBlank => self.stat.vblank_interrupt(),
            PpuMode::OamScan => self.stat.oam_interrupt(),
            PpuMode::Drawing => false,
        };

        if trigger {
            self.stat_interrupt = true;
        }
    }

    /// Check LY == LYC and potentially trigger STAT interrupt
    fn check_lyc(&mut self) {
        let coincidence = self.ly == self.lyc;
        self.stat.set_coincidence(coincidence);

        if coincidence && self.stat.lyc_interrupt() {
            self.stat_interrupt = true;
        }
    }

    /// Render one scanline to the framebuffer
    fn render_scanline(&mut self) {
        let ly = self.ly as usize;
        if ly >= SCREEN_HEIGHT {
            return;
        }

        // Clear scanline
        let line_start = ly * SCREEN_WIDTH;
        for x in 0..SCREEN_WIDTH {
            self.framebuffer[line_start + x] = 0;
        }

        // Render background
        if self.lcdc.bg_enable() {
            self.render_background(ly);
        }

        // Render window
        if self.lcdc.window_enable() && self.wy <= self.ly {
            self.render_window(ly);
        }

        // Render sprites
        if self.lcdc.obj_enable() {
            self.render_sprites(ly);
        }
    }

    /// Render background for one scanline
    fn render_background(&mut self, ly: usize) {
        let tile_map_base = if self.lcdc.bg_tile_map() { 0x1C00 } else { 0x1800 };
        let tile_data_base = if self.lcdc.bg_window_tile_data() { 0x0000 } else { 0x0800 };
        let signed_tile = !self.lcdc.bg_window_tile_data();

        let y = ((ly as u16 + self.scy as u16) & 0xFF) as u8;
        let tile_row = (y / 8) as u16;
        let tile_y = y % 8;

        let line_start = ly * SCREEN_WIDTH;

        for screen_x in 0..SCREEN_WIDTH {
            let x = ((screen_x as u16 + self.scx as u16) & 0xFF) as u8;
            let tile_col = (x / 8) as u16;
            let tile_x = x % 8;

            let tile_map_addr = tile_map_base + tile_row * 32 + tile_col;
            let tile_num = self.vram[tile_map_addr as usize];

            let tile_addr = if signed_tile {
                let signed_tile = tile_num as i8 as i16;
                (tile_data_base as i16 + (signed_tile + 128) * 16) as u16
            } else {
                tile_data_base + tile_num as u16 * 16
            };

            let color = self.get_tile_pixel(tile_addr, tile_x, tile_y);
            let palette_color = self.apply_palette(color, self.bgp);

            self.framebuffer[line_start + screen_x] = palette_color;
        }
    }

    /// Render window for one scanline
    fn render_window(&mut self, ly: usize) {
        // Window X is offset by 7
        let wx = self.wx.saturating_sub(7);

        // Check if window is visible on this line
        if wx >= SCREEN_WIDTH as u8 {
            return;
        }
        if !self.window_triggered && self.wy == self.ly {
            self.window_triggered = true;
        }
        if !self.window_triggered {
            return;
        }

        let tile_map_base = if self.lcdc.window_tile_map() { 0x1C00 } else { 0x1800 };
        let tile_data_base = if self.lcdc.bg_window_tile_data() { 0x0000 } else { 0x0800 };
        let signed_tile = !self.lcdc.bg_window_tile_data();

        let window_y = self.window_line;
        let tile_row = (window_y / 8) as u16;
        let tile_y = window_y % 8;

        let line_start = ly * SCREEN_WIDTH;

        for screen_x in (wx as usize)..SCREEN_WIDTH {
            let window_x = (screen_x - wx as usize) as u8;
            let tile_col = (window_x / 8) as u16;
            let tile_x = window_x % 8;

            let tile_map_addr = tile_map_base + tile_row * 32 + tile_col;
            let tile_num = self.vram[tile_map_addr as usize];

            let tile_addr = if signed_tile {
                let signed_tile = tile_num as i8 as i16;
                (tile_data_base as i16 + (signed_tile + 128) * 16) as u16
            } else {
                tile_data_base + tile_num as u16 * 16
            };

            let color = self.get_tile_pixel(tile_addr, tile_x, tile_y);
            let palette_color = self.apply_palette(color, self.bgp);

            self.framebuffer[line_start + screen_x] = palette_color;
        }

        self.window_line += 1;
    }

    /// Render sprites for one scanline
    fn render_sprites(&mut self, ly: usize) {
        let sprite_height = if self.lcdc.obj_size() { 16 } else { 8 };
        let ly_i16 = ly as i16;

        // Collect sprites on this scanline (max 10)
        let mut sprites_on_line: Vec<(u8, Sprite)> = Vec::with_capacity(10);

        for i in 0..40 {
            let sprite = self.get_sprite(i);
            let sprite_y = sprite.y as i16 - 16;

            if ly_i16 >= sprite_y && ly_i16 < sprite_y + sprite_height as i16 {
                sprites_on_line.push((i as u8, sprite));
                if sprites_on_line.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by X coordinate (lower X = higher priority), then by OAM index
        sprites_on_line.sort_by(|a, b| {
            if a.1.x == b.1.x {
                a.0.cmp(&b.0)
            } else {
                a.1.x.cmp(&b.1.x)
            }
        });

        let line_start = ly * SCREEN_WIDTH;

        // Render sprites in reverse order (lowest priority first, so higher priority overwrites)
        for (_, sprite) in sprites_on_line.iter().rev() {
            let sprite_x = sprite.x as i16 - 8;
            let sprite_y = sprite.y as i16 - 16;

            let mut tile_y = (ly_i16 - sprite_y) as u8;
            if sprite.y_flip() {
                tile_y = sprite_height - 1 - tile_y;
            }

            let tile_num = if sprite_height == 16 {
                if tile_y >= 8 {
                    sprite.tile | 0x01
                } else {
                    sprite.tile & 0xFE
                }
            } else {
                sprite.tile
            };

            let tile_y_in_tile = tile_y % 8;
            let tile_addr = tile_num as u16 * 16;

            for tile_x in 0..8 {
                let screen_x = sprite_x + tile_x as i16;
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as i16 {
                    continue;
                }

                let actual_tile_x = if sprite.x_flip() { 7 - tile_x } else { tile_x };
                let color = self.get_tile_pixel(tile_addr, actual_tile_x, tile_y_in_tile);

                // Color 0 is transparent for sprites
                if color == 0 {
                    continue;
                }

                let screen_x = screen_x as usize;
                let bg_color = self.framebuffer[line_start + screen_x];

                // Check sprite priority
                if sprite.priority() && bg_color != 0 {
                    continue;
                }

                let palette = if sprite.palette() { self.obp1 } else { self.obp0 };
                let palette_color = self.apply_palette(color, palette);

                self.framebuffer[line_start + screen_x] = palette_color;
            }
        }
    }

    /// Get a pixel from a tile (2bpp format)
    fn get_tile_pixel(&self, tile_addr: u16, x: u8, y: u8) -> u8 {
        let addr = tile_addr + (y as u16 * 2);
        let low = self.vram[addr as usize];
        let high = self.vram[(addr + 1) as usize];

        let bit = 7 - x;
        let color_low = (low >> bit) & 1;
        let color_high = (high >> bit) & 1;

        (color_high << 1) | color_low
    }

    /// Apply palette to get final color
    fn apply_palette(&self, color: u8, palette: u8) -> u8 {
        (palette >> (color * 2)) & 0x03
    }

    /// Get sprite from OAM
    fn get_sprite(&self, index: usize) -> Sprite {
        let base = index * 4;
        Sprite {
            y: self.oam[base],
            x: self.oam[base + 1],
            tile: self.oam[base + 2],
            flags: self.oam[base + 3],
        }
    }

    /// Read from VRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        // During mode 3, VRAM is not accessible
        if self.mode == PpuMode::Drawing && self.lcdc.lcd_enable() {
            return 0xFF;
        }
        self.vram[(addr & 0x1FFF) as usize]
    }

    /// Write to VRAM
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        if self.mode == PpuMode::Drawing && self.lcdc.lcd_enable() {
            return;
        }
        self.vram[(addr & 0x1FFF) as usize] = value;
    }

    /// Read from OAM
    pub fn read_oam(&self, addr: u16) -> u8 {
        // During mode 2 and 3, OAM is not accessible
        if (self.mode == PpuMode::OamScan || self.mode == PpuMode::Drawing) && self.lcdc.lcd_enable()
        {
            return 0xFF;
        }
        let index = (addr & 0xFF) as usize;
        if index < 160 {
            self.oam[index]
        } else {
            0xFF
        }
    }

    /// Write to OAM
    pub fn write_oam(&mut self, addr: u16, value: u8) {
        if (self.mode == PpuMode::OamScan || self.mode == PpuMode::Drawing) && self.lcdc.lcd_enable()
        {
            return;
        }
        let index = (addr & 0xFF) as usize;
        if index < 160 {
            self.oam[index] = value;
        }
    }

    /// Read PPU register
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc.0,
            0xFF41 => self.stat.0 | 0x80, // Bit 7 always 1
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    /// Write PPU register
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => {
                let was_enabled = self.lcdc.lcd_enable();
                self.lcdc.0 = value;
                // When LCD is turned off, reset PPU state
                if was_enabled && !self.lcdc.lcd_enable() {
                    self.ly = 0;
                    self.dot = 0;
                    self.mode = PpuMode::HBlank;
                    self.stat.set_mode(0);
                    self.window_line = 0;
                    self.window_triggered = false;
                }
            }
            0xFF41 => {
                // Lower 3 bits are read-only
                self.stat.0 = (self.stat.0 & 0x07) | (value & 0xF8);
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // LY is read-only
            0xFF45 => {
                self.lyc = value;
                self.check_lyc();
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_modes() {
        let mut ppu = Ppu::new();
        ppu.lcdc.0 = 0x91; // LCD on

        // Initial mode should be OAM scan
        assert_eq!(ppu.mode, PpuMode::OamScan);

        // After 80 dots, should be in Drawing mode
        ppu.tick(80);
        assert_eq!(ppu.mode, PpuMode::Drawing);

        // After 172 more dots, should be in HBlank
        ppu.tick(172);
        assert_eq!(ppu.mode, PpuMode::HBlank);
    }

    #[test]
    fn test_ly_increment() {
        let mut ppu = Ppu::new();
        ppu.lcdc.0 = 0x91;

        assert_eq!(ppu.ly, 0);

        // Complete one scanline (456 dots)
        ppu.tick(456);
        assert_eq!(ppu.ly, 1);
    }

    #[test]
    fn test_vblank_interrupt() {
        let mut ppu = Ppu::new();
        ppu.lcdc.0 = 0x91;

        // Run through 144 scanlines
        for _ in 0..(144 * 456) {
            ppu.tick(1);
        }

        assert!(ppu.vblank_interrupt);
        assert_eq!(ppu.mode, PpuMode::VBlank);
    }

    #[test]
    fn test_tile_pixel() {
        let mut ppu = Ppu::new();

        // Write a simple tile pattern
        // Each row is 2 bytes (low, high)
        ppu.vram[0] = 0xFF; // Low byte: 11111111
        ppu.vram[1] = 0x00; // High byte: 00000000
        // Result: all pixels are color 1

        for x in 0..8 {
            let color = ppu.get_tile_pixel(0, x, 0);
            assert_eq!(color, 1);
        }

        // Different pattern
        ppu.vram[0] = 0x00;
        ppu.vram[1] = 0xFF;
        // Result: all pixels are color 2

        for x in 0..8 {
            let color = ppu.get_tile_pixel(0, x, 0);
            assert_eq!(color, 2);
        }
    }

    #[test]
    fn test_palette() {
        let ppu = Ppu::new();

        // Default BGP palette: 0xFC = 11 11 11 00
        // Color 0 -> 00 (white)
        // Color 1 -> 11 (black)
        // Color 2 -> 11 (black)
        // Color 3 -> 11 (black)
        assert_eq!(ppu.apply_palette(0, 0xFC), 0);
        assert_eq!(ppu.apply_palette(1, 0xFC), 3);
        assert_eq!(ppu.apply_palette(2, 0xFC), 3);
        assert_eq!(ppu.apply_palette(3, 0xFC), 3);

        // E4 = 11 10 01 00
        assert_eq!(ppu.apply_palette(0, 0xE4), 0);
        assert_eq!(ppu.apply_palette(1, 0xE4), 1);
        assert_eq!(ppu.apply_palette(2, 0xE4), 2);
        assert_eq!(ppu.apply_palette(3, 0xE4), 3);
    }
}
