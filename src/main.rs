#![allow(dead_code)]
mod cpu8085;
mod memory;
mod ioport;
mod asm8085;

use asm8085::*;
use memory::Memory;
use cpu8085::PP8085;

fn main() {
    let (bin, listing) = parse("examples/led1.asm");
    let rom = Memory::new_from(&bin, 1024*8);
    let mut cpu = PP8085::new();

    println!("{}", listing);

    cpu.load_memory(rom);
    cpu.run();
    cpu.display();
}
