// Interrupts
//
// The Game Boy has 5 interrupt sources, each with a fixed vector address:
//
// | Bit | Interrupt | Vector | Priority |
// |-----|-----------|--------|----------|
// |  0  | V-Blank   | 0x0040 | Highest  |
// |  1  | LCD STAT  | 0x0048 |          |
// |  2  | Timer     | 0x0050 |          |
// |  3  | Serial    | 0x0058 |          |
// |  4  | Joypad    | 0x0060 | Lowest   |
//
// Registers:
//   IE (0xFFFF): Interrupt Enable - which interrupts are enabled
//   IF (0xFF0F): Interrupt Flag - which interrupts are pending
//   IME: Interrupt Master Enable (internal CPU flag, set by EI/DI)
//
// Interrupt handling:
// 1. Check if IME is true and (IE & IF) != 0
// 2. Disable IME
// 3. Push PC onto stack
// 4. Jump to interrupt vector
// 5. Clear the IF bit for the handled interrupt
//
// Special behavior:
// - EI enables interrupts after the NEXT instruction (1 instruction delay)
// - HALT wakes up when (IE & IF) != 0, even if IME is false

/// Interrupt bit flags
#[derive(Debug, Clone, Copy)]
pub struct InterruptFlags {
    pub vblank: bool,   // Bit 0
    pub lcd_stat: bool, // Bit 1
    pub timer: bool,    // Bit 2
    pub serial: bool,   // Bit 3
    pub joypad: bool,   // Bit 4
}

impl InterruptFlags {
    pub fn new() -> Self {
        Self {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }

    /// Convert to byte (for IF/IE registers)
    pub fn to_byte(&self) -> u8 {
        let mut val = 0u8;
        if self.vblank { val |= 0x01; }
        if self.lcd_stat { val |= 0x02; }
        if self.timer { val |= 0x04; }
        if self.serial { val |= 0x08; }
        if self.joypad { val |= 0x10; }
        val
    }

    /// Set from byte
    pub fn from_byte(&mut self, val: u8) {
        self.vblank = (val & 0x01) != 0;
        self.lcd_stat = (val & 0x02) != 0;
        self.timer = (val & 0x04) != 0;
        self.serial = (val & 0x08) != 0;
        self.joypad = (val & 0x10) != 0;
    }
}

impl Default for InterruptFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Interrupt vectors
pub const VBLANK_VECTOR: u16 = 0x0040;
pub const LCD_STAT_VECTOR: u16 = 0x0048;
pub const TIMER_VECTOR: u16 = 0x0050;
pub const SERIAL_VECTOR: u16 = 0x0058;
pub const JOYPAD_VECTOR: u16 = 0x0060;

/// Get the vector address for the highest priority pending interrupt
pub fn get_interrupt_vector(ie: u8, if_reg: u8) -> Option<(u16, u8)> {
    let pending = ie & if_reg;

    if pending & 0x01 != 0 {
        Some((VBLANK_VECTOR, 0x01))
    } else if pending & 0x02 != 0 {
        Some((LCD_STAT_VECTOR, 0x02))
    } else if pending & 0x04 != 0 {
        Some((TIMER_VECTOR, 0x04))
    } else if pending & 0x08 != 0 {
        Some((SERIAL_VECTOR, 0x08))
    } else if pending & 0x10 != 0 {
        Some((JOYPAD_VECTOR, 0x10))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_flags_conversion() {
        let mut flags = InterruptFlags::new();

        flags.vblank = true;
        flags.timer = true;
        assert_eq!(flags.to_byte(), 0x05);

        flags.from_byte(0x1F);
        assert!(flags.vblank);
        assert!(flags.lcd_stat);
        assert!(flags.timer);
        assert!(flags.serial);
        assert!(flags.joypad);
    }

    #[test]
    fn test_interrupt_priority() {
        // V-Blank has highest priority
        let ie = 0x1F; // All enabled
        let if_reg = 0x05; // V-Blank and Timer pending

        let (vector, bit) = get_interrupt_vector(ie, if_reg).unwrap();
        assert_eq!(vector, VBLANK_VECTOR);
        assert_eq!(bit, 0x01);
    }

    #[test]
    fn test_interrupt_masking() {
        let ie = 0x04; // Only Timer enabled
        let if_reg = 0x03; // V-Blank and LCD STAT pending (but not enabled)

        // No interrupt should fire because none are enabled
        assert!(get_interrupt_vector(ie, if_reg).is_none());

        // Now with Timer also pending
        let if_reg = 0x07;
        let (vector, _) = get_interrupt_vector(ie, if_reg).unwrap();
        assert_eq!(vector, TIMER_VECTOR);
    }
}
