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

## Architecture (To Be Implemented)

The emulator will consist of these core components:
- **CPU (SM83)**: Sharp LR35902 - Z80-like processor with unique opcodes
- **Memory Bus**: Memory mapping and access (ROM, RAM, I/O registers)
- **PPU**: Pixel Processing Unit for graphics rendering
- **APU**: Audio Processing Unit for sound
- **Timer**: DIV, TIMA, TMA, TAC registers
- **Joypad**: Input handling
- **Cartridge**: ROM loading and MBC (Memory Bank Controller) support
