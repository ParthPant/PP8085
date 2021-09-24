#![allow(dead_code)]
mod cpu8085;
mod memory;
mod ioport;
mod asm8085;

use asm8085::*;

fn main() {
    parse("examples/sertest.asm");
}