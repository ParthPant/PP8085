import {PP8085, Memory, parse_wasm} from "wasm-pp8085";

const cpu = PP8085.new();
const rom = Memory.new(1024*8);

var prg = `
; COMMENT DESCRIPTION
          MVI A, 5dh
NEXT:     DCR A
          JNZ NEXT 
          HLT
`
const bin = parse_wasm(prg);
for (let i = 0; i<bin.length; i++) {
    rom.write(i, bin[i]);
}

cpu.load_memory(rom);
cpu.run();
console.log(cpu.get_summary());