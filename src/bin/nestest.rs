use std::fs;

use nes_emulator::bus::CpuBus;
use nes_emulator::cpu::trace;
use nes_emulator::{cartridge, cpu};

fn main() {
    let file_name = "nestest/nestest.nes";

    let raw = fs::read(file_name).expect("nestest.nes not found");

    let cartridge = cartridge::Cartridge::new(&raw);
    let bus = CpuBus::new(cartridge);

    let mut cpu = cpu::CPU::new(bus);
    cpu.reset().expect("Could not reset CPU.");

    cpu.program_counter = 0xc000;

    cpu.run_with_callback(|cpu| {
        trace::trace(cpu).expect("Error producing trace");
    })
    .expect("Error running cpu");
}
