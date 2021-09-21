#![allow(dead_code)]
mod cpu8085;
mod memory;
mod ioport;

use cpu8085::PP8085;

fn main() {
    let mut cpu = PP8085::new();
}