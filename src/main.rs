// Game Boy Emulator in Rust
//
// This emulator implements the original Game Boy (DMG - Dot Matrix Game)
// Hardware specifications:
//   CPU: Sharp SM83 (LR35902) - 8-bit, 4.194304 MHz
//   RAM: 8KB Work RAM + 127 bytes High RAM
//   VRAM: 8KB Video RAM
//   Display: 160x144 pixels, 4 shades of gray
//   Sound: 4 channels (2 pulse, 1 wave, 1 noise)

mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod interrupts;
mod timer;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use emulator::Emulator;
use std::env;

fn main() {
    println!("Game Boy Emulator");
    println!("=================\n");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <rom_file> [--run] [--debug]", args[0]);
        println!("  --run    Execute the ROM (default: just show info)");
        println!("  --debug  Show debug output during execution");
        println!("\nRunning in demo mode...\n");
        run_demo();
        return;
    }

    let rom_path = &args[1];
    let run_mode = args.iter().any(|a| a == "--run");
    let debug_mode = args.iter().any(|a| a == "--debug");

    match Cartridge::from_file(rom_path) {
        Ok(cart) => {
            println!("ROM loaded: {}", rom_path);
            println!("  Title: {}", cart.info.title);
            println!("  Type: {:?}", cart.info.cartridge_type);
            println!("  ROM size: {}KB", cart.info.rom_size / 1024);
            println!("  RAM size: {}KB", cart.info.ram_size / 1024);
            println!(
                "  Header checksum: 0x{:02X} ({})",
                cart.info.header_checksum,
                if cart.info.checksum_valid { "valid" } else { "INVALID" }
            );

            if run_mode {
                run_rom(&cart, debug_mode);
            } else {
                // Just show ROM info and first bytes
                let bus = Bus::new();
                println!("\nFirst instructions at 0x0100:");
                for i in 0..16 {
                    let addr = 0x0100 + i;
                    if i < cart.rom.len() as u16 {
                        print!("{:02X} ", cart.rom[addr as usize]);
                    }
                    if i == 7 {
                        println!();
                    }
                }
                println!();
                println!("\nUse --run to execute the ROM");
            }
        }
        Err(e) => {
            eprintln!("Error loading ROM: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run a ROM file
fn run_rom(cart: &Cartridge, debug: bool) {
    println!("\n--- Executing ROM ---\n");

    let mut emu = Emulator::new(cart);

    // Maximum cycles to run (about 1200 seconds of emulated time)
    // 4.194304 MHz * 1200 seconds = ~5 billion cycles
    let max_cycles: u64 = 5_000_000_000;

    let mut last_output_len = 0;
    let mut instructions_executed = 0u64;

    while emu.cycles < max_cycles {
        if debug && instructions_executed % 100_000 == 0 {
            let ie = emu.bus.read(0xFFFF);
            let if_reg = emu.bus.read(0xFF0F);
            println!(
                "[{:>10} cycles] PC: 0x{:04X}, A: 0x{:02X}, IE: 0x{:02X}, IF: 0x{:02X}, IME: {}, HALT: {}",
                emu.cycles, emu.cpu.regs.pc, emu.cpu.regs.a, ie, if_reg, emu.cpu.ime, emu.cpu.halted
            );
        }

        emu.step();
        instructions_executed += 1;

        // Check for new serial output
        let output = emu.get_serial_output();
        if output.len() > last_output_len {
            let new_chars = &output[last_output_len..];
            print!("{}", new_chars);
            last_output_len = output.len();

            // Check for test completion
            if output.contains("Passed") || output.contains("Failed") {
                println!();
                break;
            }
        }

        // Safety check for infinite loops without output
        if instructions_executed > 500_000_000 {
            println!("\n[Timeout: 500M instructions without completion]");
            break;
        }
    }

    println!("\n--- Execution Summary ---");
    println!("  Instructions: {}", instructions_executed);
    println!("  Cycles: {}", emu.cycles);
    println!("  CPU halted: {}", emu.cpu.halted);

    let output = emu.get_serial_output();
    if !output.is_empty() {
        println!("\n--- Serial Output ---");
        println!("{}", output);
    }

    if emu.test_passed() {
        println!("\n[TEST PASSED]");
    } else if emu.test_failed() {
        println!("\n[TEST FAILED]");
    }
}

fn run_demo() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    println!("CPU initialized:");
    println!("  PC: 0x{:04X}", cpu.regs.pc);
    println!("  SP: 0x{:04X}", cpu.regs.sp);
    println!("  A: 0x{:02X}, F: 0x{:02X}", cpu.regs.a, cpu.regs.f.to_byte());
    println!("  BC: 0x{:04X}", cpu.regs.bc());
    println!("  DE: 0x{:04X}", cpu.regs.de());
    println!("  HL: 0x{:04X}", cpu.regs.hl());

    println!("\n--- CPU Instruction Demo ---");
    println!("Loading test program into ROM at 0x0100...\n");

    // Simple test program:
    // 0x0100: LD A, 0x00      ; A = 0
    // 0x0102: LD B, 0x05      ; B = 5 (loop counter)
    // 0x0104: INC A           ; A++
    // 0x0105: DEC B           ; B--
    // 0x0106: JR NZ, -4       ; if B != 0, jump back to INC A
    // 0x0108: HALT            ; Stop

    let program: &[u8] = &[
        0x3E, 0x00,  // LD A, 0x00
        0x06, 0x05,  // LD B, 0x05
        0x3C,        // INC A
        0x05,        // DEC B
        0x20, 0xFC,  // JR NZ, -4 (0xFC = -4 as signed byte)
        0x76,        // HALT
    ];

    // Load program at 0x0100
    bus.load_rom(&{
        let mut rom = vec![0u8; 0x8000];
        for (i, byte) in program.iter().enumerate() {
            rom[0x0100 + i] = *byte;
        }
        rom
    });

    println!("Program loaded. Executing...\n");
    println!("{:^6} {:^6} {:^6} {:^6} {:^6} {:^10}", "PC", "A", "B", "F", "Cycles", "Instruction");
    println!("{:-<6} {:-<6} {:-<6} {:-<6} {:-<6} {:-<10}", "", "", "", "", "", "");

    let mut total_cycles = 0u32;
    let instructions = [
        "LD A, 0x00",
        "LD B, 0x05",
        "INC A", "DEC B", "JR NZ, -4",
        "INC A", "DEC B", "JR NZ, -4",
        "INC A", "DEC B", "JR NZ, -4",
        "INC A", "DEC B", "JR NZ, -4",
        "INC A", "DEC B", "JR NZ, -4",
        "HALT",
    ];
    let mut inst_idx = 0;

    while !cpu.halted && inst_idx < instructions.len() {
        let pc_before = cpu.regs.pc;
        let cycles = cpu.step(&mut bus);
        total_cycles += cycles;

        println!(
            "0x{:04X} 0x{:02X}   0x{:02X}   0x{:02X}   {:>4}   {}",
            pc_before,
            cpu.regs.a,
            cpu.regs.b,
            cpu.regs.f.to_byte(),
            cycles,
            instructions[inst_idx]
        );
        inst_idx += 1;
    }

    println!("\nExecution complete!");
    println!("  Total cycles: {}", total_cycles);
    println!("  Final A: 0x{:02X} ({})", cpu.regs.a, cpu.regs.a);
    println!("  Final B: 0x{:02X}", cpu.regs.b);
    println!("  CPU halted: {}", cpu.halted);
}
