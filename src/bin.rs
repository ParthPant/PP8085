use pp8085_lib::memory::Memory;
use pp8085_lib::cpu8085::PP8085;
use pp8085_lib::asm8085::*;

fn main() {
    let (bin, listing) = parse("examples/test.asm");
    let rom = Memory::new_from(&bin, 1024*8);
    let mut cpu = PP8085::new();

    println!("{}", listing);

    cpu.load_memory(rom);
    cpu.run();
    cpu.display();
}
