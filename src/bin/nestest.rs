use std::fs;

use nes_emulator::bus::Bus;
use nes_emulator::cpu::trace;
use nes_emulator::{cartridge, cpu};

fn main() {
    let file_name = "nestest/nestest.nes";

    let raw = fs::read(file_name).expect("nestest.nes not found");

    let cartridge = cartridge::Cartridge::new(&raw);
    let bus = Bus::new(cartridge);

    let mut cpu = cpu::CPU::new(bus);
    cpu.reset();

    cpu.program_counter = 0xc000;

    cpu.run_with_callback(|cpu| {
        trace::trace(cpu);
    });
}
