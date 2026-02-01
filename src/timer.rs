// Timer
//
// The Game Boy has a timer system with 4 registers:
//
// DIV  (0xFF04): Divider Register - Increments at 16384 Hz (every 256 cycles)
//                Writing any value resets it to 0
//
// TIMA (0xFF05): Timer Counter - Increments at frequency specified by TAC
//                When it overflows (>0xFF), it's reset to TMA and
//                a Timer interrupt is requested
//
// TMA  (0xFF06): Timer Modulo - Value loaded into TIMA on overflow
//
// TAC  (0xFF07): Timer Control
//                Bit 2: Timer Enable (0=Stop, 1=Start)
//                Bits 1-0: Clock Select
//                  00: 4096 Hz   (every 1024 cycles)
//                  01: 262144 Hz (every 16 cycles)
//                  10: 65536 Hz  (every 64 cycles)
//                  11: 16384 Hz  (every 256 cycles)
//
// Internal counter:
// The timer uses a 16-bit internal counter. DIV is the upper 8 bits.
// TIMA increments based on specific bits of this counter.

/// Timer state
pub struct Timer {
    /// Internal 16-bit counter (DIV is upper 8 bits)
    /// Increments every T-cycle
    internal_counter: u16,
    /// TIMA - Timer Counter (0xFF05)
    pub tima: u8,
    /// TMA - Timer Modulo (0xFF06)
    pub tma: u8,
    /// TAC - Timer Control (0xFF07)
    pub tac: u8,
    /// Interrupt request flag
    pub interrupt_requested: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            internal_counter: 0xABCC, // Post-boot value
            tima: 0,
            tma: 0,
            tac: 0,
            interrupt_requested: false,
        }
    }

    /// Get DIV register (upper 8 bits of internal counter)
    pub fn div(&self) -> u8 {
        (self.internal_counter >> 8) as u8
    }

    /// Reset DIV (writing any value to DIV resets it)
    pub fn reset_div(&mut self) {
        // Resetting DIV can trigger TIMA increment if the selected bit goes from 1 to 0
        let old_bit = self.get_timer_bit();
        self.internal_counter = 0;
        let new_bit = self.get_timer_bit();

        // Falling edge detection
        if old_bit && !new_bit && self.timer_enabled() {
            self.increment_tima();
        }
    }

    /// Check if timer is enabled
    fn timer_enabled(&self) -> bool {
        (self.tac & 0x04) != 0
    }

    /// Get the bit of internal counter that controls TIMA increments
    fn get_timer_bit(&self) -> bool {
        let bit_pos = match self.tac & 0x03 {
            0 => 9,  // 4096 Hz (bit 9)
            1 => 3,  // 262144 Hz (bit 3)
            2 => 5,  // 65536 Hz (bit 5)
            3 => 7,  // 16384 Hz (bit 7)
            _ => unreachable!(),
        };
        (self.internal_counter & (1 << bit_pos)) != 0
    }

    /// Increment TIMA, handling overflow
    fn increment_tima(&mut self) {
        let (new_tima, overflow) = self.tima.overflowing_add(1);
        if overflow {
            self.tima = self.tma;
            self.interrupt_requested = true;
        } else {
            self.tima = new_tima;
        }
    }

    /// Update timer state for elapsed cycles
    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
            let old_bit = self.get_timer_bit() && self.timer_enabled();

            self.internal_counter = self.internal_counter.wrapping_add(1);

            let new_bit = self.get_timer_bit() && self.timer_enabled();

            // Falling edge detection: bit goes from 1 to 0
            if old_bit && !new_bit {
                self.increment_tima();
            }
        }
    }

    /// Write to TAC register
    pub fn write_tac(&mut self, value: u8) {
        let old_bit = self.get_timer_bit() && self.timer_enabled();
        self.tac = value;
        let new_bit = self.get_timer_bit() && self.timer_enabled();

        // Changing TAC can trigger TIMA increment
        if old_bit && !new_bit {
            self.increment_tima();
        }
    }

    /// Take the interrupt request (clears the flag)
    pub fn take_interrupt(&mut self) -> bool {
        let requested = self.interrupt_requested;
        self.interrupt_requested = false;
        requested
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_increment() {
        let mut timer = Timer::new();
        timer.internal_counter = 0;

        // DIV increments every 256 cycles
        timer.tick(256);
        assert_eq!(timer.div(), 1);

        timer.tick(256);
        assert_eq!(timer.div(), 2);
    }

    #[test]
    fn test_div_reset() {
        let mut timer = Timer::new();
        timer.internal_counter = 0x1234;

        timer.reset_div();
        assert_eq!(timer.div(), 0);
        assert_eq!(timer.internal_counter, 0);
    }

    #[test]
    fn test_tima_disabled() {
        let mut timer = Timer::new();
        timer.internal_counter = 0;
        timer.tac = 0x00; // Timer disabled

        timer.tick(10000);
        assert_eq!(timer.tima, 0); // TIMA should not increment
    }

    #[test]
    fn test_tima_overflow() {
        let mut timer = Timer::new();
        timer.internal_counter = 0;
        timer.tima = 0xFF;
        timer.tma = 0x42;
        timer.tac = 0x05; // Enabled, clock select 01 (fastest)

        // Should overflow after 16 cycles
        timer.tick(16);

        assert_eq!(timer.tima, 0x42); // Reset to TMA
        assert!(timer.interrupt_requested);
    }

    #[test]
    fn test_timer_frequency() {
        // Test clock select 01 (262144 Hz = every 16 cycles)
        let mut timer = Timer::new();
        timer.internal_counter = 0;
        timer.tima = 0;
        timer.tac = 0x05; // Enabled, clock 01

        timer.tick(16);
        assert_eq!(timer.tima, 1);

        timer.tick(16);
        assert_eq!(timer.tima, 2);
    }
}
