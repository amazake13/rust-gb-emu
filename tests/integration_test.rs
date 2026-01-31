// Integration tests for the Game Boy emulator

use rust_gb_emu::emulator::Emulator;

/// Create a ROM with a test program
fn create_test_rom(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];

    // Copy program to entry point
    for (i, byte) in program.iter().enumerate() {
        rom[0x0100 + i] = *byte;
    }

    rom
}

#[test]
fn test_serial_hello_world() {
    // Program that outputs "Hello" via serial
    let program: &[u8] = &[
        // Output "Hello" via serial (0xFF01 = SB, 0xFF02 = SC)
        0x3E, b'H',       // LD A, 'H'
        0xE0, 0x01,       // LDH (0x01), A  -> SB
        0x3E, 0x81,       // LD A, 0x81
        0xE0, 0x02,       // LDH (0x02), A  -> SC (trigger transfer)

        0x3E, b'e',       // LD A, 'e'
        0xE0, 0x01,
        0x3E, 0x81,
        0xE0, 0x02,

        0x3E, b'l',       // LD A, 'l'
        0xE0, 0x01,
        0x3E, 0x81,
        0xE0, 0x02,

        0x3E, b'l',       // LD A, 'l'
        0xE0, 0x01,
        0x3E, 0x81,
        0xE0, 0x02,

        0x3E, b'o',       // LD A, 'o'
        0xE0, 0x01,
        0x3E, 0x81,
        0xE0, 0x02,

        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(100_000);

    assert_eq!(emu.get_serial_output(), "Hello");
    assert!(emu.cpu.halted);
}

#[test]
fn test_add_instruction() {
    // Test ADD A, B instruction
    // A = 0x10, B = 0x20, result should be 0x30
    let program: &[u8] = &[
        0x3E, 0x10,       // LD A, 0x10
        0x06, 0x20,       // LD B, 0x20
        0x80,             // ADD A, B
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x30);
    assert!(!emu.cpu.regs.f.z); // Not zero
    assert!(!emu.cpu.regs.f.n); // Addition
    assert!(!emu.cpu.regs.f.h); // No half carry
    assert!(!emu.cpu.regs.f.c); // No carry
}

#[test]
fn test_add_with_carry() {
    // Test ADD that causes carry
    // A = 0xFF, B = 0x01, result should be 0x00 with carry
    let program: &[u8] = &[
        0x3E, 0xFF,       // LD A, 0xFF
        0x06, 0x01,       // LD B, 0x01
        0x80,             // ADD A, B
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x00);
    assert!(emu.cpu.regs.f.z); // Zero
    assert!(emu.cpu.regs.f.h); // Half carry (0x0F + 1 = 0x10)
    assert!(emu.cpu.regs.f.c); // Carry
}

#[test]
fn test_sub_instruction() {
    // Test SUB instruction
    let program: &[u8] = &[
        0x3E, 0x30,       // LD A, 0x30
        0x06, 0x10,       // LD B, 0x10
        0x90,             // SUB B
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x20);
    assert!(emu.cpu.regs.f.n); // Subtraction flag
}

#[test]
fn test_loop_counter() {
    // Count from 0 to 10 using a loop
    let program: &[u8] = &[
        0x3E, 0x00,       // LD A, 0x00
        0x06, 0x0A,       // LD B, 0x0A (10)
        // loop:
        0x3C,             // INC A
        0x05,             // DEC B
        0x20, 0xFC,       // JR NZ, -4 (back to INC A)
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(10_000);

    assert_eq!(emu.cpu.regs.a, 10);
    assert_eq!(emu.cpu.regs.b, 0);
}

#[test]
fn test_push_pop() {
    // Test PUSH and POP
    let program: &[u8] = &[
        0x01, 0x34, 0x12, // LD BC, 0x1234
        0xC5,             // PUSH BC
        0x01, 0x00, 0x00, // LD BC, 0x0000 (clear BC)
        0xD1,             // POP DE (should get 0x1234)
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.de(), 0x1234);
}

#[test]
fn test_call_ret() {
    // Test CALL and RET
    let program: &[u8] = &[
        // Main: CALL subroutine, then HALT
        0xCD, 0x08, 0x01, // CALL 0x0108 (subroutine)
        0x76,             // HALT (0x0103)

        // Padding to reach 0x0108
        0x00, 0x00, 0x00, 0x00,

        // Subroutine at 0x0108:
        0x3E, 0x42,       // LD A, 0x42
        0xC9,             // RET
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x42);
    assert_eq!(emu.cpu.regs.pc, 0x0104); // After HALT
}

#[test]
fn test_bit_operations() {
    // Test BIT, SET, RES
    let program: &[u8] = &[
        0x3E, 0x00,       // LD A, 0x00
        0xCB, 0xC7,       // SET 0, A  -> A = 0x01
        0xCB, 0xCF,       // SET 1, A  -> A = 0x03
        0xCB, 0xD7,       // SET 2, A  -> A = 0x07
        0xCB, 0x87,       // RES 0, A  -> A = 0x06
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x06);
}

#[test]
fn test_rotate() {
    // Test RLCA
    let program: &[u8] = &[
        0x3E, 0x85,       // LD A, 0x85 (1000_0101)
        0x07,             // RLCA -> 0x0B (0000_1011), C=1
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0x0B);
    assert!(emu.cpu.regs.f.c);
}

#[test]
fn test_swap() {
    // Test SWAP
    let program: &[u8] = &[
        0x3E, 0xAB,       // LD A, 0xAB
        0xCB, 0x37,       // SWAP A -> 0xBA
        0x76,             // HALT
    ];

    let rom = create_test_rom(program);
    let mut emu = Emulator::with_rom(&rom);

    emu.run_until_halt(1000);

    assert_eq!(emu.cpu.regs.a, 0xBA);
}
