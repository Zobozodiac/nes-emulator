use nes_emulator;

fn main() {
    println!("Hello, world!");

    let signed_int: i8 = -1;
    let unsigned_int = signed_int as u8;

    println!("unsigned_int: {:b}", unsigned_int);

    let unsigned_int: u8 = 0b1111_1111;
    let signed_int = unsigned_int as i8;

    println!("signed_int: {}", signed_int);

    println!("signed_int u16: {:b}", signed_int as u16);

    let memory: u16 = 0xffff;

    println!("memory: {}", memory);
    println!("adding -1: {}", memory.wrapping_add(signed_int as u16));

    println!("relative: {}", (0xf8 as u8) as i8);

    let mut main_val = 0;

    let x = [1, 2, 3];

    for val in x {
        println!("val: {}", main_val);
        main_val += 1;
    }
}
