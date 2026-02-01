# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust implementation of a Game Boy (DMG) emulator. Development follows an incremental approach: implement one component at a time, verify with test ROMs before moving on.

## Build Commands

```bash
cargo build              # Build the project
cargo run                # Run the emulator
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo clippy             # Run linter
cargo fmt                # Format code

# Run with ROM
cargo run -- path/to/rom.gb --run
cargo run -- path/to/rom.gb --run --debug
```

## Reference Documentation

- Pan Docs: https://bgb.bircd.org/pandocs.htm (primary spec reference)
- Opcodes: https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html (note: some cycle counts are incorrect)
- Verify implementations against test ROMs rather than trusting documentation blindly

## Development Philosophy

1. **Incremental implementation**: Build one component at a time
2. **Test-driven**: Verify each component with test ROMs before proceeding
3. **Document specs**: Record which Game Boy specifications have been implemented
4. **Explain implementation**: Add explanations for complex hardware behaviors

## Architecture

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library exports
├── bus.rs           # Memory bus (address mapping)
├── cpu/
│   ├── mod.rs       # CPU structure
│   ├── registers.rs # CPU registers (A,F,B,C,D,E,H,L,SP,PC)
│   ├── instructions.rs    # Base opcodes (0x00-0xFF)
│   └── cb_instructions.rs # CB-prefixed opcodes
├── cartridge.rs     # ROM loading and header parsing
├── emulator.rs      # Main emulation loop
├── interrupts.rs    # Interrupt handling
└── timer.rs         # Timer (DIV, TIMA, TMA, TAC)
```

## Implemented Specifications

### CPU (SM83) ✅
- All 256 base opcodes
- All 256 CB-prefixed opcodes (bit operations)
- Correct flag handling (Z, N, H, C)
- Verified with Blargg's cpu_instrs (11/11 tests pass)

### Memory Bus ✅
- Full 64KB address space mapping
- ROM (0x0000-0x7FFF)
- VRAM (0x8000-0x9FFF)
- External RAM (0xA000-0xBFFF)
- WRAM (0xC000-0xDFFF)
- Echo RAM (0xE000-0xFDFF)
- OAM (0xFE00-0xFE9F)
- I/O registers (0xFF00-0xFF7F)
- HRAM (0xFF80-0xFFFE)
- IE register (0xFFFF)

### Interrupts ✅
- 5 interrupt sources (V-Blank, LCD STAT, Timer, Serial, Joypad)
- IE/IF registers
- IME flag with EI/DI control
- EI instruction 1-cycle delay
- HALT and wake on interrupt

### Timer ✅
- DIV register (0xFF04) - 16384 Hz
- TIMA counter (0xFF05)
- TMA modulo (0xFF06)
- TAC control (0xFF07)
- Timer interrupt on overflow

### Cartridge ✅
- ROM loading
- Header parsing (title, type, sizes)
- Header checksum validation
- MBC type detection (ROM Only, MBC1-5)

### Serial ✅
- Basic serial output capture (for test ROMs)

## Not Yet Implemented

- **PPU**: Graphics rendering (background, window, sprites)
- **APU**: Audio (4 channels)
- **Joypad**: Input handling
- **MBC**: Bank switching (MBC1, MBC3, MBC5)
- **CGB**: Color Game Boy features
