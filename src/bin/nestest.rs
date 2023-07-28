use nes_emulator::bus::Bus;
use nes_emulator::{cartridge, cpu};
use std::fs;
use std::rc::Rc;

fn main() {
    let file_name = "nestest/nestest.nes";

    let raw = fs::read(file_name).expect("nestest.nes not found");

    let cartridge = cartridge::Cartridge::new(&raw);
    let bus = Bus::new(cartridge);

    let mut cpu = cpu::CPU::new(bus);

    cpu.run_with_callback(|cpu| {
        println!("Iteration:");
        println!("register_a: {:#04x}", cpu.register_a);
        println!("register_x: {:#04x}", cpu.register_x);
        println!("register_y: {:#04x}", cpu.register_y);
        println!("stack_pointer: {:#04x}", cpu.stack_pointer);
        println!("program_counter: {:#06x}", cpu.program_counter);
        println!("status_byte: {:b}", cpu.status.get_status_byte());
        println!();
    });
}
