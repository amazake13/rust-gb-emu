// Joypad Input
//
// The Game Boy has 8 buttons read through register 0xFF00:
//   - D-pad: Right, Left, Up, Down
//   - Buttons: A, B, Select, Start
//
// Register 0xFF00 (P1/JOYP):
//   Bit 7-6: Unused (return 1)
//   Bit 5: Select Action buttons (0 = select)
//   Bit 4: Select Direction buttons (0 = select)
//   Bit 3: Down or Start (0 = pressed)
//   Bit 2: Up or Select (0 = pressed)
//   Bit 1: Left or B (0 = pressed)
//   Bit 0: Right or A (0 = pressed)
//
// Reading works by:
//   1. Writing to select which button group (bits 4-5)
//   2. Reading to get button states (bits 0-3)
//
// Note: 0 = pressed, 1 = not pressed (active low)

/// Joypad state
#[derive(Debug, Clone, Copy, Default)]
pub struct Joypad {
    /// Direction buttons (active low internally)
    /// Bit 0: Right, Bit 1: Left, Bit 2: Up, Bit 3: Down
    directions: u8,
    /// Action buttons (active low internally)
    /// Bit 0: A, Bit 1: B, Bit 2: Select, Bit 3: Start
    actions: u8,
    /// Button group selection
    /// Bit 4: Select directions, Bit 5: Select actions
    select: u8,
    /// Joypad interrupt pending
    pub interrupt: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            directions: 0x0F, // All released (1 = not pressed)
            actions: 0x0F,    // All released
            select: 0x30,     // Neither group selected
            interrupt: false,
        }
    }

    /// Read the joypad register (0xFF00)
    pub fn read(&self) -> u8 {
        let mut result = 0xCF; // Bits 7-6 always 1, bits 3-0 start as 1

        // Check which button group is selected (active low)
        if self.select & 0x10 == 0 {
            // Direction buttons selected
            result = (result & 0xF0) | (self.directions & 0x0F);
        }
        if self.select & 0x20 == 0 {
            // Action buttons selected
            result = (result & 0xF0) | (self.actions & 0x0F);
        }

        // Include selection bits
        result = (result & 0x0F) | (self.select & 0x30) | 0xC0;

        result
    }

    /// Write to the joypad register (0xFF00)
    /// Only bits 4-5 are writable (button group selection)
    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    /// Press a button
    pub fn press(&mut self, button: Button) {
        let old_state = self.read() & 0x0F;

        match button {
            Button::Right => self.directions &= !0x01,
            Button::Left => self.directions &= !0x02,
            Button::Up => self.directions &= !0x04,
            Button::Down => self.directions &= !0x08,
            Button::A => self.actions &= !0x01,
            Button::B => self.actions &= !0x02,
            Button::Select => self.actions &= !0x04,
            Button::Start => self.actions &= !0x08,
        }

        // Check if any button went from high to low (interrupt condition)
        let new_state = self.read() & 0x0F;
        if old_state != 0x0F && new_state < old_state {
            self.interrupt = true;
        }
    }

    /// Release a button
    pub fn release(&mut self, button: Button) {
        match button {
            Button::Right => self.directions |= 0x01,
            Button::Left => self.directions |= 0x02,
            Button::Up => self.directions |= 0x04,
            Button::Down => self.directions |= 0x08,
            Button::A => self.actions |= 0x01,
            Button::B => self.actions |= 0x02,
            Button::Select => self.actions |= 0x04,
            Button::Start => self.actions |= 0x08,
        }
    }

    /// Update button state (true = pressed)
    pub fn set_button(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.press(button);
        } else {
            self.release(button);
        }
    }

    /// Take the interrupt flag (returns and clears it)
    pub fn take_interrupt(&mut self) -> bool {
        let result = self.interrupt;
        self.interrupt = false;
        result
    }
}

/// Button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let joypad = Joypad::new();
        // With no group selected, should return 0xFF (all high)
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn test_direction_buttons() {
        let mut joypad = Joypad::new();

        // Select direction buttons
        joypad.write(0x20); // Bit 4 = 0, Bit 5 = 1

        // All released
        assert_eq!(joypad.read() & 0x0F, 0x0F);

        // Press Right
        joypad.press(Button::Right);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Bit 0 = 0

        // Press Up
        joypad.press(Button::Up);
        assert_eq!(joypad.read() & 0x0F, 0x0A); // Bits 0 and 2 = 0

        // Release Right
        joypad.release(Button::Right);
        assert_eq!(joypad.read() & 0x0F, 0x0B); // Bit 2 = 0
    }

    #[test]
    fn test_action_buttons() {
        let mut joypad = Joypad::new();

        // Select action buttons
        joypad.write(0x10); // Bit 4 = 1, Bit 5 = 0

        // Press A
        joypad.press(Button::A);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Bit 0 = 0

        // Press Start
        joypad.press(Button::Start);
        assert_eq!(joypad.read() & 0x0F, 0x06); // Bits 0 and 3 = 0
    }

    #[test]
    fn test_both_groups() {
        let mut joypad = Joypad::new();

        // Press Right (direction) and A (action)
        joypad.press(Button::Right);
        joypad.press(Button::A);

        // Select directions only
        joypad.write(0x20);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Only Right shows

        // Select actions only
        joypad.write(0x10);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Only A shows

        // Select both (unusual but possible)
        joypad.write(0x00);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Both show (AND together)
    }
}
