import {PP8085, Memory, parse_wasm} from "wasm-pp8085";

const cpu = PP8085.new();

var prg = `
; COMMENT DESCRIPTION
          MVI A, 5dh
NEXT:     DCR A
          JNZ NEXT 
          HLT
`
const bin = parse_wasm(prg);
const rom = Memory.new_from_js(bin, 1024*8);
console.log(rom.get_copy());
cpu.load_memory(rom);
cpu.run();
console.log(cpu.get_summary());