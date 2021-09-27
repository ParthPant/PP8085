use pp8085_lib::memory::Memory;
use pp8085_lib::cpu8085::PP8085;
use pp8085_lib::asm8085::*;

fn main() {
    let (bin1, listing1) = parse("examples/test.asm").unwrap();
    let rom = Memory::new_from(&bin1, 1024*8);
    let mut cpu = PP8085::new();
    cpu.load_memory(rom);
    while !cpu.get_hlt() {
        cpu.run_next();
        cpu.display();
    }
}
