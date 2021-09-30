use super::memory::Memory;
use super::ioport::IoPort;
use std::collections::HashMap;
use std::fmt;
use wasm_bindgen::prelude::*;

#[allow(non_snake_case)]
#[wasm_bindgen]
pub struct PP8085 {
    IR:u8, // Instruction Register
    A: u8, // Accumulator
    F: u8, // Process Status Register
    
    // Registers
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,

    PC:u16, // Program Counter Register
    SP:u16, // Stack Pointer

    memory: Memory,
    io_ports: HashMap<u8, IoPort>,

    cycles: u32,
    IE: bool,  // Interrupt enable
    HLT: bool, // indicates hlt state
}

impl fmt::Display for PP8085 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A:{}\tF:{}\nB:{}\tC:{}\nD:{}\tE:{}\nH:{}\tL:{}\nPC:{}\tSP{}\n",
        self.A, self.F, self.B, self.C, self.D, self.E, self.H, self.L, self.PC, self.SP)
    }
}

#[wasm_bindgen]
impl PP8085 {
    /// creates a new cpu and initializes everything to zero.
    pub fn new() -> PP8085 {
        console_error_panic_hook::set_once();
        PP8085 {
            IR:0,  // Instruction Register
            A: 0,  // Accumulator
            F: 0,  // Process Status Register

            // Registers
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,

            PC:0,  // Program Counter Register
            SP:0, // Stack Pointer

            memory: Memory::new(8192),
            io_ports: HashMap::new(),

            cycles: 0,
            IE: false,
            HLT: false,
        }
    }

    /// execution cycle
    pub fn run(&mut self) {
        while !self.HLT {
            if self.cycles == 0 {
                self.IR = self.read_8bits();
                self.cycles += self.decode_and_run() as u32;
            }
            self.cycles -= 1;
        }
    }

    /// execute one instruction and stop with no regard to cycles 
    pub fn run_next(&mut self) {
        if !self.HLT {
            self.IR = self.read_8bits();
            self.decode_and_run();
        }
    }

    pub fn add_io_port(&mut self, addr: u8) {
        self.io_ports.insert(addr, IoPort::new(addr));
    }

    pub fn remove_io_port(&mut self, addr: u8) {
        self.io_ports.remove(&addr);
    }

    pub fn load_memory(&mut self, data: Memory) {
        self.memory = data;
    }

    pub fn read_io(&mut self, addr: u8) -> u8 {
        match self.io_ports.get(&addr) {
            Some(d) => d.read(),
            None => 0,
        }
    }

    pub fn write_io(&mut self, addr: u8, data: u8) {
        if let Some(port) = self.io_ports.get_mut(&addr) {
            port.write(data);
        };
    }

    pub fn get_summary(&self) -> String {
        self.to_string()
    }
    
    pub fn get_memory_ptr(&self) -> *const u8 {
        self.memory.get_data()
    }

    pub fn get_io_ports(&self) -> JsValue {
        JsValue::from_serde(&self.io_ports).unwrap()
    }

    pub fn get_a(&self) -> u8 {self.A}
    pub fn get_f(&self) -> u8 {self.F}
    pub fn get_b(&self) -> u8 {self.B}
    pub fn get_c(&self) -> u8 {self.C}
    pub fn get_d(&self) -> u8 {self.D}
    pub fn get_e(&self) -> u8 {self.E}
    pub fn get_h(&self) -> u8 {self.H}
    pub fn get_l(&self) -> u8 {self.L}
    pub fn get_sp(&self) -> u16 {self.SP}
    pub fn get_pc(&self) -> u16 {self.PC}
    pub fn get_ir(&self) -> u8 {self.IR}
    pub fn get_hlt(&self) -> bool {self.HLT}
    
    pub fn reset(&mut self) {
        self.A = 0; self.B = 0; self.C = 0; self.D = 0; self.E = 0; self.H = 0; self.L = 0; self.SP = 0; self.PC = 0; self.F = 0;
        self.IR = 0;
        self.HLT = false;
        self.cycles = 0;
    }
}

macro_rules!  mov_rd_rs {
    ($fn_name: ident, $dest: ident, $source: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.$dest = self.$source;
            4
        }
    };
}

macro_rules!  mov_m_rs {
    ($fn_name: ident, $source: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.write_memory(self.get_addr_hl(), self.$source);
            7
        }
    };
}

macro_rules!  mov_rd_m {
    ($fn_name: ident, $dest: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.$dest = self.read_memory(self.get_addr_hl());
            7
        }
    };
}

macro_rules! inr_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            if self.$r < 0xff {
                self.$r += 1;
            } else {
                self.$r = 0x00;
            }
            let num = self.$r;
            self.set_sign((num | 1<<7) != 0);
            self.set_overflow(num == 0x00);
            self.set_zero(num == 0x00);
            if num != 0x00 {
                self.set_auxiliary_carry((((num-1) & 0x0f) + 0x01) & 0x10 == 0x10);
            } else {
                self.set_auxiliary_carry(true);
            }
            self.set_parity(PP8085::find_parity(num));
            4
        }
    };
}

macro_rules! dcr_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            if self.$r > 0x00 {
                self.$r -= 1;
            } else {
                self.$r = 0xff;
            }
            let num = self.$r;
            self.set_sign((num | 1<<7) != 0);
            self.set_overflow(num == 0xff);
            self.set_zero(num == 0x00);
            if num != 0xff {
                self.set_auxiliary_carry(((num+1) & 0x0f) < 1);
            } else {
                self.set_auxiliary_carry(true);
            }
            self.set_parity(PP8085::find_parity(num));
            4
        }
    };
}

macro_rules! add_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let num = self.$r;
            self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
            self.set_carry(0xff - self.A < num);
            if 0xff - self.A > num {
                self.A += num; 
            } else {
                self.A = num - (0xff - self.A + 0x01);
                self.set_overflow(true);
            }
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! adc_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let num = self.$r + (self.F & 1);
            self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
            self.set_carry(0xff - self.A < num);
            if 0xff - self.A > num {
                self.A += num; 
            } else {
                self.A = num - (0xff - self.A + 0x01);
                self.set_overflow(true);
            }
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! sub_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let num = self.$r;
            self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
            if self.A >= num {
                self.A -= num;
            } else {
                self.A = 0xff - (num - self.A - 0x01);
                self.set_carry(true);
                self.set_overflow(true);
            }
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! sbb_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let num = self.$r + (self.F & 1);
            self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
            if self.A >= num {
                self.A -= num;
            } else {
                self.A = 0xff - (num - self.A - 0x01);
                self.set_carry(true);
                self.set_overflow(true);
            }
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! ana_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.A &= self.$r;
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! xra_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.A ^= self.$r;
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! ora_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.A |= self.$r;
            self.set_sign((self.A | 1<<7) != 0);
            self.set_zero(self.A == 0x00);
            self.set_parity(PP8085::find_parity(self.A));
            4
        }
    }
}

macro_rules! cmp_r {
    ($fn_name: ident, $r: ident) => {
        fn $fn_name (&mut self) -> u8 {
            match self.$r.cmp(&self.A) {
                std::cmp::Ordering::Equal => {
                    self.set_carry(false);
                    self.set_zero(true);
                }
                std::cmp::Ordering::Greater => {
                    self.set_carry(false);
                    self.set_zero(false);
                }
                std::cmp::Ordering::Less => {
                    self.set_carry(true);
                    self.set_zero(false);
                }
            }
            4
        }
    };
}

macro_rules! rst_seq {
    ($fn_name: ident, $i: expr) => {
        fn $fn_name (&mut self) -> u8 {
            self.SP -= 1;
            self.write_memory(self.SP, (self.PC >> 8) as u8);
            self.SP -= 1;
            self.write_memory(self.SP, (self.PC & 0x0f) as u8);
            self.PC = $i as u16;
            12 
        }
    };
}

macro_rules! call_seq {
    ($fn_name: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let (opl, oph) = self.read_16bits(); 
            self.SP -= 1;
            self.write_memory(self.SP, (self.PC >> 8) as u8);
            self.SP -= 1;
            self.write_memory(self.SP, (self.PC & 0x0f) as u8);
            self.PC = ((oph as u16) << 8) | (opl as u16);
            18
        }
    };

    ($fn_name1: ident, $fn_name2: ident, $cond: ident) => {
        fn $fn_name1 (&mut self) -> u8 {
            let (opl, oph) = self.read_16bits(); 
            if self.$cond() {
                self.SP -= 1;
                self.write_memory(self.SP, (self.PC >> 8) as u8);
                self.SP -= 1;
                self.write_memory(self.SP, (self.PC & 0x0f) as u8);
                self.PC = ((oph as u16) << 8) | (opl as u16);
                return 18;
            }
            9
        }

        fn $fn_name2 (&mut self) -> u8 {
            let (opl, oph) = self.read_16bits(); 
            if !self.$cond() {
                self.SP -= 1;
                self.write_memory(self.SP, (self.PC >> 8) as u8);
                self.SP -= 1;
                self.write_memory(self.SP, (self.PC & 0x0f) as u8);
                self.PC = ((oph as u16) << 8) | (opl as u16);
                return 18;
            }
            9
        }
    };
}

macro_rules! ret_seq {
    ($fn_name: ident) => {
        fn $fn_name (&mut self) -> u8 {
            let l = self.read_memory(self.SP);
            self.SP += 1;
            let h = self.read_memory(self.SP);
            self.SP += 1;
            self.PC = ((h as u16) << 8) | (l as u16);
            10
        }
    };

    ($fn_name1: ident, $fn_name2: ident, $cond: ident) => {
        fn $fn_name1 (&mut self) -> u8 {
            if self.$cond() {
                let l = self.read_memory(self.SP);
                self.SP += 1;
                let h = self.read_memory(self.SP);
                self.SP += 1;
                self.PC = ((h as u16) << 8) | (l as u16);
                return 12;
            }
            6
        }

        fn $fn_name2 (&mut self) -> u8 {
            if !self.$cond() {
                let l = self.read_memory(self.SP);
                self.SP += 1;
                let h = self.read_memory(self.SP);
                self.SP += 1;
                self.PC = ((h as u16) << 8) | (l as u16);
                return 12;
            }
            6
        }
    };
}

macro_rules! dad_p {
    ($fn_name: ident,$a: ident, $b: ident) => {
       fn $fn_name(&mut self) -> u8 {
           let mut hl = self.get_addr_hl();
           let ab = ((self.$a as u16) << 8) | (self.$b as u16);
           if 0xffff - hl > ab {
               hl += ab;
               self.set_carry(false);
               self.set_overflow(false);
           } else {
               hl = ab - (0xffff - hl + 0x0001);
               self.set_carry(true);
               self.set_overflow(true);
           }
           self.H = (hl >> 8) as u8;
           self.L = (hl & 0x0f) as u8;
           10
       } 
    };

    ($fn_name: ident) => {
       fn $fn_name(&mut self) -> u8 {
           let mut hl = self.get_addr_hl();
           let ab = self.SP;
           if 0xffff - hl > ab {
               hl += ab;
               self.set_carry(false);
               self.set_overflow(false);
           } else {
               hl = ab - (0xffff - hl + 0x0001);
               self.set_carry(true);
               self.set_overflow(true);
           }
           self.H = (hl >> 8) as u8;
           self.L = (hl & 0x0f) as u8;
           10
       } 
    }
}

impl PP8085 {
    /// display the contents of all the registers.
    pub fn display(&self) {
        println!("A : {:#02x}\tF : {:#02x}", self.A, self.F);
        println!("B : {:#02x}\tC : {:#02x}", self.B, self.C);
        println!("D : {:#02x}\tE : {:#02x}", self.D, self.E);
        println!("H : {:#02x}\tL : {:#02x}", self.H, self.L);
        println!("PC: {:#04x}", self.PC);
        println!("SP: {:#04x}", self.SP);
        println!("-----------------------------");
    }

    fn decode_and_run(&mut self) -> u8 {
        match self.IR {
            0x00 => self.nop(),
            0x01 => self.lxi_b(),
            0x02 => self.stax_b(),
            0x03 => self.inx_b(),
            0x04 => self.inr_b(),
            0x05 => self.dcr_b(),
            0x06 => self.mvi_b(),
            0x07 => self.rlc(),
            0x09 => self.dad_b(),
            0x0A => self.ldax_b(),
            0x0B => self.dcx_b(),
            0x0C => self.inr_c(),
            0x0D => self.dcr_c(),
            0x0E => self.mvi_c(),
            0x0F => self.rrc(),
            0x11 => self.lxi_d(),
            0x12 => self.stax_d(),
            0x13 => self.inx_d(),
            0x14 => self.inr_d(),
            0x15 => self.dcr_d(),
            0x16 => self.mvi_d(),
            0x17 => self.ral(),
            0x19 => self.dad_d(),
            0x1A => self.ldax_d(),
            0x1B => self.dcx_d(),
            0x1C => self.inr_e(),
            0x1D => self.dcr_e(),
            0x1E => self.mvi_e(),
            0x1F => self.rar(),
            0x20 => self.rim(),
            0x21 => self.lxi_h(),
            0x22 => self.shld(),
            0x23 => self.inx_h(),
            0x24 => self.inr_h(),
            0x25 => self.dcr_h(),
            0x26 => self.mvi_h(),
            0x27 => self.daa(),
            0x29 => self.dad_h(),
            0x2A => self.lhld(),
            0x2B => self.dcx_h(),
            0x2C => self.inr_l(),
            0x2D => self.dcr_l(),
            0x2E => self.mvi_l(),
            0x2F => self.cma(),
            0x30 => self.sim(),
            0x31 => self.lxi_sp(),
            0x32 => self.sta(),
            0x33 => self.inx_sp(),
            0x34 => self.inr_m(),
            0x35 => self.dcr_m(),
            0x36 => self.mvi_m(),
            0x37 => self.stc(),
            0x39 => self.dad_sp(),
            0x3A => self.lda(),
            0x3B => self.dcx_sp(),
            0x3C => self.inr_a(),
            0x3D => self.dcr_a(),
            0x3E => self.mvi_a(),
            0x3F => self.cmc(),
            0x40 => self.mov_b_b(),
            0x41 => self.mov_b_c(),
            0x42 => self.mov_b_d(),
            0x43 => self.mov_b_e(),
            0x44 => self.mov_b_h(),
            0x45 => self.mov_b_l(),
            0x46 => self.mov_b_m(),
            0x47 => self.mov_b_a(),
            0x48 => self.mov_c_b(),
            0x49 => self.mov_c_c(),
            0x4A => self.mov_c_d(),
            0x4B => self.mov_c_e(),
            0x4C => self.mov_c_h(),
            0x4D => self.mov_c_l(),
            0x4E => self.mov_c_m(),
            0x4F => self.mov_c_a(),
            0x50 => self.mov_d_b(),
            0x51 => self.mov_d_c(),
            0x52 => self.mov_d_d(),
            0x53 => self.mov_d_e(),
            0x54 => self.mov_d_h(),
            0x55 => self.mov_d_l(),
            0x56 => self.mov_d_m(),
            0x57 => self.mov_d_a(),
            0x58 => self.mov_e_b(),
            0x59 => self.mov_e_c(),
            0x5A => self.mov_e_d(),
            0x5B => self.mov_e_e(),
            0x5C => self.mov_e_h(),
            0x5D => self.mov_e_l(),
            0x5E => self.mov_e_m(),
            0x5F => self.mov_e_a(),
            0x60 => self.mov_h_b(),
            0x61 => self.mov_h_c(),
            0x62 => self.mov_h_d(),
            0x63 => self.mov_h_e(),
            0x64 => self.mov_h_h(),
            0x65 => self.mov_h_l(),
            0x66 => self.mov_h_m(),
            0x67 => self.mov_h_a(),
            0x68 => self.mov_l_b(),
            0x69 => self.mov_l_c(),
            0x6A => self.mov_l_d(),
            0x6B => self.mov_l_e(),
            0x6C => self.mov_l_h(),
            0x6D => self.mov_l_l(),
            0x6E => self.mov_l_m(),
            0x6F => self.mov_l_a(),
            0x70 => self.mov_m_b(),
            0x71 => self.mov_m_c(),
            0x72 => self.mov_m_d(),
            0x73 => self.mov_m_e(),
            0x74 => self.mov_m_h(),
            0x75 => self.mov_m_l(),
            0x76 => self.hlt(),
            0x77 => self.mov_m_a(),
            0x78 => self.mov_a_b(),
            0x79 => self.mov_a_c(),
            0x7A => self.mov_a_d(),
            0x7B => self.mov_a_e(),
            0x7C => self.mov_a_h(),
            0x7D => self.mov_a_l(),
            0x7E => self.mov_a_m(),
            0x7F => self.mov_a_a(),
            0x80 => self.add_b(),
            0x81 => self.add_c(),
            0x82 => self.add_d(),
            0x83 => self.add_e(),
            0x84 => self.add_h(),
            0x85 => self.add_l(),
            0x86 => self.add_m(),
            0x87 => self.add_a(),
            0x88 => self.adc_b(),
            0x89 => self.adc_c(),
            0x8A => self.adc_d(),
            0x8B => self.adc_e(),
            0x8C => self.adc_h(),
            0x8D => self.adc_l(),
            0x8E => self.adc_m(),
            0x8F => self.adc_a(),
            0x90 => self.sub_b(),
            0x91 => self.sub_c(),
            0x92 => self.sub_d(),
            0x93 => self.sub_e(),
            0x94 => self.sub_h(),
            0x95 => self.sub_l(),
            0x96 => self.sub_m(),
            0x97 => self.sub_a(),
            0x98 => self.sbb_b(),
            0x99 => self.sbb_c(),
            0x9A => self.sbb_d(),
            0x9B => self.sbb_e(),
            0x9C => self.sbb_h(),
            0x9D => self.sbb_l(),
            0x9E => self.sbb_m(),
            0x9F => self.sbb_a(),
            0xA0 => self.ana_b(),
            0xA1 => self.ana_c(),
            0xA2 => self.ana_d(),
            0xA3 => self.ana_e(),
            0xA4 => self.ana_h(),
            0xA5 => self.ana_l(),
            0xA6 => self.ana_m(),
            0xA7 => self.ana_a(),
            0xA8 => self.xra_b(),
            0xA9 => self.xra_c(),
            0xAA => self.xra_d(),
            0xAB => self.xra_e(),
            0xAC => self.xra_h(),
            0xAD => self.xra_l(),
            0xAE => self.xra_m(),
            0xAF => self.xra_a(),
            0xB0 => self.ora_b(),
            0xB1 => self.ora_c(),
            0xB2 => self.ora_d(),
            0xB3 => self.ora_e(),
            0xB4 => self.ora_h(),
            0xB5 => self.ora_l(),
            0xB6 => self.ora_m(),
            0xB7 => self.ora_a(),
            0xB8 => self.cmp_b(),
            0xB9 => self.cmp_c(),
            0xBA => self.cmp_d(),
            0xBB => self.cmp_e(),
            0xBC => self.cmp_h(),
            0xBD => self.cmp_l(),
            0xBE => self.cmp_m(),
            0xBF => self.cmp_a(),
            0xC0 => self.rnz(),
            0xC1 => self.pop_b(),
            0xC2 => self.jnz(),
            0xC3 => self.jmp(),
            0xC4 => self.cnz(),
            0xC5 => self.push_b(),
            0xC6 => self.adi(),
            0xC7 => self.rst_0(),
            0xC8 => self.rz(),
            0xC9 => self.ret(),
            0xCA => self.jz(),
            0xCC => self.cz(),
            0xCD => self.call(),
            0xCE => self.aci(),
            0xCF => self.rst_1(),
            0xD0 => self.rnc(),
            0xD1 => self.pop_d(),
            0xD2 => self.jnc(),
            0xD3 => self.out(),
            0xD4 => self.cnc(),
            0xD5 => self.push_d(),
            0xD6 => self.sui(),
            0xD7 => self.rst_2(),
            0xD8 => self.rc(),
            0xDA => self.jc(),
            0xDB => self.i_n(),
            0xDC => self.cc(),
            0xDE => self.sbi(),
            0xDF => self.rst_3(),
            0xE0 => self.rpo(),
            0xE1 => self.pop_h(),
            0xE2 => self.jpo(),
            0xE3 => self.xthl(),
            0xE4 => self.cpo(),
            0xE5 => self.push_h(),
            0xE6 => self.ani(),
            0xE7 => self.rst_4(),
            0xE8 => self.rpe(),
            0xE9 => self.pchl(),
            0xEA => self.jpe(),
            0xEB => self.xchg(),
            0xEC => self.cpe(),
            0xEE => self.xri(),
            0xEF => self.rst_5(),
            0xF0 => self.rp(),
            0xF1 => self.pop_psw(),
            0xF2 => self.jp(),
            0xF3 => self.di(),
            0xF4 => self.cp(),
            0xF5 => self.push_psw(),
            0xF6 => self.ori(),
            0xF7 => self.rst_6(),
            0xF8 => self.rm(),
            0xF9 => self.sphl(),
            0xFA => self.jm(),
            0xFB => self.ei(),
            0xFC => self.cm(),
            0xFE => self.cpi(),
            0xFF => self.rst_7(),
            _ => {panic!("{:#02x} is unimplemented", self.IR)}
        }
    }

    fn write_memory(&mut self, addr: u16, content: u8) {
        self.memory.write(addr, content);
    }

    pub fn read_memory(&mut self, addr: u16) -> u8 {
        self.memory.read(addr)
    }

    /// return parity flag
    fn find_parity(x: u8) -> bool {
        let x = x as u32;
        let mut y: u32 = x ^ (x >> 1);
        y = y ^ (y >> 2);
        y = y ^ (y >> 4);
        y = y ^ (y >> 8);
        y = y ^ (y >> 16);

        if y & 1 != 0{
            return false;
        }
        true
    }

    // utility functions to set flags according to the results.
    // 7  6  5  4  3  2  1  0
    // S  Z (K) A  0  P (V) C
    fn set_sign(&mut self, b: bool) {
        if b {
            self.F |= 1<<7;
        } else {
            self.F &= !(1<<7);
        }
    }

    fn get_sign(&self) -> bool {
        return (self.F & 1<<7) != 0x0
    }

    fn set_zero(&mut self, b: bool) {
        if b {
            self.F |= 1<<6;
        } else {
            self.F &= !(1<<6);
        }
    }

    fn get_zero(&self) -> bool {
        return (self.F & 1<<6) != 0x0
    }

    fn set_overflow(&mut self, b: bool) {
        if b {
            self.F |= 1<<5;
            self.F |= 1<<1;
        } else {
            self.F &= !(1<<5);
            self.F &= !(1<<1);
        }
    }

    fn get_overflow(&self) -> bool {
        return (self.F & 1<<5) != 0x0
    }

    fn set_auxiliary_carry(&mut self, b: bool) {
        if b {
            self.F |= 1<<4;
        } else {
            self.F &= !(1<<4);
        }
    }

    fn get_auxiliary_carry(&self) -> bool {
        return (self.F & 1<<4) != 0x0
    }

    fn set_parity(&mut self, b: bool) {
        if b {
            self.F |= 1<<2;
        } else {
            self.F &= !(1<<2);
        }
    }

    fn get_parity(&self) -> bool {
        return (self.F & 1<<2) != 0x0
    }

    fn set_carry(&mut self, b: bool) {
        if b {
            self.F |= 1<<0;
        } else {
            self.F &= !(1<<0);
        }
    }

    fn get_carry(&self) -> bool {
        return (self.F & 1<<0) != 0x0
    }

    /// return the address stored in HL register pair as a u16.
    fn get_addr_hl(&self) -> u16 {
        (self.H as u16) << 8 | self.L as u16
    }
    
    /// return the address stored in BC register pair as a u16.
    fn get_addr_bc(&self) -> u16 {
        (self.B as u16) << 8 | self.C as u16
    }

    /// return the address stored in DE register pair as a u16.
    fn get_addr_de(&self) -> u16 {
        (self.D as u16) << 8 | self.E as u16
    }

    fn read_8bits(&mut self) ->  u8 {
        let r = self.read_memory(self.PC);
        self.PC += 1;
        r
    }

    fn read_16bits(&mut self) ->  (u8, u8) {
        let l = self.read_memory(self.PC);
        self.PC += 1;
        let h = self.read_memory(self.PC);
        self.PC += 1;
        (l, h)
    }

    /// NOP
    fn nop(&mut self) -> u8 {
        4
    }

    // EI
    fn ei(&mut self) -> u8 {
        self.IE = true;
        4
    }

    // DI
    fn di(&mut self) -> u8 {
        self.IE = false;
        4
    }

    // HLT
    fn hlt(&mut self) -> u8 {
        self.HLT = true;
        5
    }

    // MOV Rs, RD instructions
    mov_rd_rs!(mov_a_a, A, A);
    mov_rd_rs!(mov_a_b, A, B);
    mov_rd_rs!(mov_a_c, A, C);
    mov_rd_rs!(mov_a_d, A, D);
    mov_rd_rs!(mov_a_e, A, E);
    mov_rd_rs!(mov_a_h, A, H);
    mov_rd_rs!(mov_a_l, A, L);

    mov_rd_rs!(mov_b_a, B, A);
    mov_rd_rs!(mov_b_b, B, B);
    mov_rd_rs!(mov_b_c, B, C);
    mov_rd_rs!(mov_b_d, B, D);
    mov_rd_rs!(mov_b_e, B, E);
    mov_rd_rs!(mov_b_h, B, H);
    mov_rd_rs!(mov_b_l, B, L);

    mov_rd_rs!(mov_c_a, C, A);
    mov_rd_rs!(mov_c_b, C, B);
    mov_rd_rs!(mov_c_c, C, C);
    mov_rd_rs!(mov_c_d, C, D);
    mov_rd_rs!(mov_c_e, C, E);
    mov_rd_rs!(mov_c_h, C, H);
    mov_rd_rs!(mov_c_l, C, L);

    mov_rd_rs!(mov_d_a, D, A);
    mov_rd_rs!(mov_d_b, D, B);
    mov_rd_rs!(mov_d_c, D, C);
    mov_rd_rs!(mov_d_d, D, D);
    mov_rd_rs!(mov_d_e, D, E);
    mov_rd_rs!(mov_d_h, D, H);
    mov_rd_rs!(mov_d_l, D, L);

    mov_rd_rs!(mov_e_a, E, A);
    mov_rd_rs!(mov_e_b, E, B);
    mov_rd_rs!(mov_e_c, E, C);
    mov_rd_rs!(mov_e_d, E, D);
    mov_rd_rs!(mov_e_e, E, E);
    mov_rd_rs!(mov_e_h, E, H);
    mov_rd_rs!(mov_e_l, E, L);

    mov_rd_rs!(mov_h_a, H, A);
    mov_rd_rs!(mov_h_b, H, B);
    mov_rd_rs!(mov_h_c, H, C);
    mov_rd_rs!(mov_h_d, H, D);
    mov_rd_rs!(mov_h_e, H, E);
    mov_rd_rs!(mov_h_h, H, H);
    mov_rd_rs!(mov_h_l, H, L);

    mov_rd_rs!(mov_l_a, L, A);
    mov_rd_rs!(mov_l_b, L, B);
    mov_rd_rs!(mov_l_c, L, C);
    mov_rd_rs!(mov_l_d, L, D);
    mov_rd_rs!(mov_l_e, L, E);
    mov_rd_rs!(mov_l_h, L, H);
    mov_rd_rs!(mov_l_l, L, L);

    // MOV M Rs
    mov_m_rs!(mov_m_a, A);
    mov_m_rs!(mov_m_b, B);
    mov_m_rs!(mov_m_c, C);
    mov_m_rs!(mov_m_d, D);
    mov_m_rs!(mov_m_e, E);
    mov_m_rs!(mov_m_h, H);
    mov_m_rs!(mov_m_l, L);

    // MOV Rd M
    mov_rd_m!(mov_a_m, A);
    mov_rd_m!(mov_b_m, B);
    mov_rd_m!(mov_c_m, C);
    mov_rd_m!(mov_d_m, D);
    mov_rd_m!(mov_e_m, E);
    mov_rd_m!(mov_h_m, H);
    mov_rd_m!(mov_l_m, L);

    /// MVI B
    fn mvi_b(&mut self) -> u8 {
        let op = self.read_8bits();
        self.B = op;
        7
    }

    /// MVI C
    fn mvi_c(&mut self) -> u8 {
        let op = self.read_8bits();
        self.C = op;
        7
    }

    /// MVI D
    fn mvi_d(&mut self) -> u8 {
        let op = self.read_8bits();
        self.D = op;
        7
    }

    /// MVI E
    fn mvi_e(&mut self) -> u8 {
        let op = self.read_8bits();
        self.E = op;
        7
    }

    /// MVI H
    fn mvi_h(&mut self) -> u8 {
        let op = self.read_8bits();
        self.H = op;
        7
    }

    /// MVI L
    fn mvi_l(&mut self) -> u8 {
        let op = self.read_8bits();
        self.L = op;
        7
    }

    /// MVI A
    fn mvi_a(&mut self) -> u8 {
        let op = self.read_8bits();
        self.A = op;
        7
    }

    /// MVI M
    fn mvi_m(&mut self) -> u8 {
        let op = self.read_8bits();
        self.write_memory(self.get_addr_hl(), op);
        10
    }

    /// LXI B
    fn lxi_b(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.B = oph;
        self.C = opl;
        3
    }

    /// LXI D
    fn lxi_d(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.D = oph;
        self.E = opl;
        3
    }

    /// LXI H
    fn lxi_h(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.H = oph;
        self.L = opl;
        3
    }

    /// LXI SP 
    fn lxi_sp(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.SP = (oph as u16) << 8 | opl as u16;
        3
    }

    /// STAX B
    /// store A indirect
    fn stax_b(&mut self) -> u8 {
        self.write_memory(self.get_addr_bc(), self.A);
        7
    }

    /// STAX D
    /// store A indirect
    fn stax_d(&mut self) -> u8 {
        self.write_memory(self.get_addr_de(), self.A);
        7
    }

    /// LDAX B
    /// load A indirect
    fn ldax_b(&mut self) -> u8 {
        self.A = self.read_memory(self.get_addr_bc());
        7
    }
    
    /// LDAX D
    /// load A indirect
    fn ldax_d(&mut self) -> u8 {
        self.A = self.read_memory(self.get_addr_de());
        7
    }

    /// STA XXXX
    /// store A direct
    fn sta(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.write_memory((oph as u16) << 8 | opl as u16, self.A);
        13 
    }
    
    /// LDA XXXX
    /// load A direct
    fn lda(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.A = self.read_memory((oph as u16) << 8 | opl as u16);
        13 
    }

    /// SHLD XXXX
    /// store HL direct
    fn shld(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        let addr = (oph as u16) << 8 | opl as u16;
        self.write_memory(addr, self.H);
        self.write_memory(addr+1, self.L);
        16 
    }
    
    /// LHLD XXXX
    /// load HL direct
    fn lhld(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        let addr = (oph as u16) << 8 | opl as u16;
        self.L = self.read_memory(addr);
        self.H = self.read_memory(addr+1);
        16 
    }

    /// XCHG
    /// exchange DE and HL registers
    fn xchg(&mut self) -> u8 {
        std::mem::swap(&mut self.H, &mut self.D);
        std::mem::swap(&mut self.L, &mut self.E);
        4
    }

    /// PUSH B
    /// push BC on stack
    fn push_b(&mut self) -> u8 {
        self.SP -= 1;
        self.write_memory(self.SP, self.B);
        self.SP -= 1;
        self.write_memory(self.SP, self.C);
        12 
    }
    
    /// PUSH D
    /// push DE on stack
    fn push_d(&mut self) -> u8 {
        self.SP -= 1;
        self.write_memory(self.SP, self.D);
        self.SP -= 1;
        self.write_memory(self.SP, self.E);
        12
    }

    /// PUSH H
    /// push HL on stack
    fn push_h(&mut self) -> u8 {
        self.SP -= 1;
        self.write_memory(self.SP, self.H);
        self.SP -= 1;
        self.write_memory(self.SP, self.L);
        12
    }

    /// PUSH PSW 
    /// push A and F on stack
    fn push_psw(&mut self) -> u8 {
        self.SP -= 1;
        self.write_memory(self.SP, self.A);
        self.SP -= 1;
        self.write_memory(self.SP, self.F);
        12
    }

    /// POP B
    /// pop BC from stack
    fn pop_b(&mut self) -> u8 {
        self.C = self.read_memory(self.SP);
        self.SP += 1;
        self.B = self.read_memory(self.SP);
        self.SP += 1;
        10
    }

    /// POP D
    /// pop DE from stack
    fn pop_d(&mut self) -> u8 {
        self.E = self.read_memory(self.SP);
        self.SP += 1;
        self.D = self.read_memory(self.SP);
        self.SP += 1;
        10
    }

    /// POP H
    /// pop HL from stack
    fn pop_h(&mut self) -> u8 {
        self.L = self.read_memory(self.SP);
        self.SP += 1;
        self.H = self.read_memory(self.SP);
        self.SP += 1;
        10
    }

    /// POP PSW
    /// pop A and F from stack
    fn pop_psw(&mut self) -> u8 {
        self.F = self.read_memory(self.SP);
        self.SP += 1;
        self.A = self.read_memory(self.SP);
        self.SP += 1;
        10
    }

    /// XTHL
    /// exchange stack with HL
    fn xthl(&mut self) -> u8 {
        let top = self.read_memory(self.SP);
        let bottom = self.read_memory(self.SP+1);

        self.write_memory(self.SP, self.L);
        self.write_memory(self.SP+1, self.H);

        self.L = top;
        self.H = bottom;
        16
    }

    /// SPHL
    /// initialize SP with HL
    fn sphl(&mut self) -> u8 {
        self.SP = self.get_addr_hl();
        6
    }

    /// INX SP
    /// increment SP by 1
    fn inx_sp(&mut self) -> u8 {
        if self.SP < 0xffff {
            self.SP += 1;
        } else {
            self.F = self.F | 1<<5;
            self.SP = 0x0000;
        }
        6
    }

    /// INX B
    /// increment register pair BC
    fn inx_b(&mut self) -> u8 {
        let mut t = self.get_addr_bc();
        if t < 0xffff {
            t += 1;
        } else {
            self.F = self.F | 1<<5;
            self.B = 0x00;
            self.C = 0x00;
            return 6;
        }
        self.B = (t >> 8) as u8;
        self.C = (t & 0x00ff) as u8;
        6
    }

    /// INX D
    /// increment register pair DE
    fn inx_d(&mut self) -> u8 {
        let mut t = self.get_addr_de();
        if t < 0xffff {
            t += 1;
        } else {
            self.F = self.F | 1<<5;
            self.D = 0x00;
            self.E = 0x00;
            return 6;
        }
        self.D = (t >> 8) as u8;
        self.E = (t & 0x00ff) as u8;
        6
    }

    /// INX H
    /// increment register pair HL
    fn inx_h(&mut self) -> u8 {
        let mut t = self.get_addr_hl();
        if t < 0xffff {
            t += 1;
        } else {
            self.F = self.F | 1<<5;
            self.H = 0x00;
            self.L = 0x00;
            return 6;
        }
        self.H = (t >> 8) as u8;
        self.L = (t & 0x00ff) as u8;
        6
    }

    /// DCX SP
    /// decrement SP by 1
    fn dcx_sp(&mut self) -> u8 {
        if self.SP > 0x0000 {
            self.SP -= 1;
        } else {
            self.F = self.F | 1<<5;
            self.SP = 0xffff;
        }
        6
    }

    /// DCX B
    /// decrement BC by 1
    fn dcx_b(&mut self) -> u8 {
        let mut t = self.get_addr_bc();
        if t > 0 {
            t -= 1;
        } else {
            self.F = self.F | 1<<5;
            self.B = 0xff;
            self.C = 0xff;
            return 6;
        }
        self.B = (t >> 8) as u8;
        self.C = (t & 0x00ff) as u8;
        6
    }

    /// DCX D
    /// decrement DE by 1
    fn dcx_d(&mut self) -> u8 {
        let mut t = self.get_addr_de();
        if t > 0 {
            t -= 1;
        } else {
            self.F = self.F | 1<<5;
            self.D = 0xff;
            self.E = 0xff;
            return 6;
        }
        self.D = (t >> 8) as u8;
        self.E = (t & 0x00ff) as u8;
        6
    }

    /// DCX H
    /// decrement HL by 1
    fn dcx_h(&mut self) -> u8 {
        let mut t = self.get_addr_hl();
        if t > 0 {
            t -= 1;
        } else {
            self.F = self.F | 1<<5;
            self.H = 0xff;
            self.L = 0xff;
            return 6;
        }
        self.H = (t >> 8) as u8;
        self.L = (t & 0x00ff) as u8;
        6
    }

    inr_r!(inr_a, A);
    inr_r!(inr_b, B);
    inr_r!(inr_c, C);
    inr_r!(inr_d, D);
    inr_r!(inr_e, E);
    inr_r!(inr_h, H);
    inr_r!(inr_l, L);

    /// INR M
    /// inrement M by 1
    fn inr_m(&mut self) -> u8 {
        let mut num = self.read_memory(self.get_addr_hl());
        if num < 0xff {
            num += 1;
            self.write_memory(self.get_addr_hl(), num);
        } else {
            num = 0x00;
            self.write_memory(self.get_addr_hl(), num);
        }
        self.set_sign((num | 1<<7) != 0);
        self.set_overflow(num == 0x00);
        self.set_zero(num == 0x00);
        if num != 0x00 {
            self.set_auxiliary_carry((((num-1) & 0x0f) + 0x01) & 0x10 == 0x10);
        } else {
            self.set_auxiliary_carry(true);
        }
        self.set_parity(PP8085::find_parity(num));
        10
    }

    dcr_r!(dcr_a, A);
    dcr_r!(dcr_b, B);
    dcr_r!(dcr_c, C);
    dcr_r!(dcr_d, D);
    dcr_r!(dcr_e, E);
    dcr_r!(dcr_h, H);
    dcr_r!(dcr_l, L);

    /// DCR M
    /// decrement M by 1
    fn dcr_m(&mut self) -> u8 {
        let mut num = self.read_memory(self.get_addr_hl());
        if num > 0x00 {
            num -= 1;
            self.write_memory(self.get_addr_hl(), num);
        } else {
            num = 0xff;
            self.write_memory(self.get_addr_hl(), num);
        }
        self.set_sign((num | 1<<7) != 0);
        self.set_overflow(num == 0xff);
        self.set_zero(num == 0x00);
        if num != 0xff {
            self.set_auxiliary_carry((((num+1) & 0x0f) - 0x01) & 0x10 == 0x10);
        } else {
            self.set_auxiliary_carry(true);
        }
        self.set_parity(PP8085::find_parity(num));
        10
    }

    add_r!(add_a, A);
    add_r!(add_b, B);
    add_r!(add_c, C);
    add_r!(add_d, D);
    add_r!(add_e, E);
    add_r!(add_h, H);
    add_r!(add_l, L);

    adc_r!(adc_a, A);
    adc_r!(adc_b, B);
    adc_r!(adc_c, C);
    adc_r!(adc_d, D);
    adc_r!(adc_e, E);
    adc_r!(adc_h, H);
    adc_r!(adc_l, L);

    /// ADD M
    fn add_m(&mut self) -> u8 {
        let num = self.read_memory(self.get_addr_hl());
        self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
        self.set_carry(0xff - self.A < num);
        if 0xff - self.A > num {
            self.A += num; 
        } else {
            self.A = num - (0xff - self.A + 0x01);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// ADC M
    fn adc_m(&mut self) -> u8 {
        let num = self.read_memory(self.get_addr_hl()) + (self.F & 1);
        self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
        self.set_carry(0xff - self.A < num);
        if 0xff - self.A > num {
            self.A += num; 
        } else {
            self.A = num - (0xff - self.A + 0x01);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// ADI
    fn adi(&mut self) -> u8 {
        let num = self.read_8bits();
        self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
        self.set_carry(0xff - self.A < num);
        if 0xff - self.A > num {
            self.A += num; 
        } else {
            self.A = num - (0xff - self.A + 0x01);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// ACI
    fn aci(&mut self) -> u8 {
        let num = self.read_8bits() + (self.F & 1);
        self.set_auxiliary_carry(((self.A & 0x0f) + (num & 0x0f)) & 0x10 == 0x10);
        self.set_carry(0xff - self.A < num);
        if 0xff - self.A > num {
            self.A += num; 
        } else {
            self.A = num - (0xff - self.A + 0x01);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    dad_p!(dad_b, B, C);
    dad_p!(dad_d, D, E);
    dad_p!(dad_h, H, L);
    dad_p!(dad_sp);

    sub_r!(sub_a, A);
    sub_r!(sub_b, B);
    sub_r!(sub_c, C);
    sub_r!(sub_d, D);
    sub_r!(sub_e, E);
    sub_r!(sub_h, H);
    sub_r!(sub_l, L);

    sbb_r!(sbb_a, A);
    sbb_r!(sbb_b, B);
    sbb_r!(sbb_c, C);
    sbb_r!(sbb_d, D);
    sbb_r!(sbb_e, E);
    sbb_r!(sbb_h, H);
    sbb_r!(sbb_l, L);

    /// SUB M
    fn sub_m(&mut self) -> u8 {
        let num = self.read_memory(self.get_addr_hl());
        self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
        if self.A >= num {
            self.A -= num;
        } else {
            self.A = 0xff - (num - self.A - 0x01);
            self.set_carry(true);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// SBB M
    fn sbb_m(&mut self) -> u8 {
        let num = self.read_memory(self.get_addr_hl()) + (self.F & 1);
        self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
        if self.A >= num {
            self.A -= num;
        } else {
            self.A = 0xff - (num - self.A - 0x01);
            self.set_carry(true);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// SUI XX
    fn sui(&mut self) -> u8 {
        let num = self.read_8bits();
        self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
        if self.A >= num {
            self.A -= num;
        } else {
            self.A = 0xff - (num - self.A - 0x01);
            self.set_carry(true);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    /// SBI XX
    fn sbi(&mut self) -> u8 {
        let num = self.read_8bits() + (self.F & 1);
        self.set_auxiliary_carry((self.A & 0x0f) < (num & 0x0f));
        if self.A >= num {
            self.A -= num;
        } else {
            self.A = 0xff - (num - self.A - 0x01);
            self.set_carry(true);
            self.set_overflow(true);
        }
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    ana_r!(ana_a, A);
    ana_r!(ana_b, B);
    ana_r!(ana_c, C);
    ana_r!(ana_d, D);
    ana_r!(ana_e, E);
    ana_r!(ana_h, H);
    ana_r!(ana_l, L);

    fn ana_m(&mut self) -> u8 {
        self.A &= self.read_memory(self.get_addr_hl()); 
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    fn ani(&mut self) -> u8 {
        self.A &= self.read_8bits(); 
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    xra_r!(xra_a, A);
    xra_r!(xra_b, B);
    xra_r!(xra_c, C);
    xra_r!(xra_d, D);
    xra_r!(xra_e, E);
    xra_r!(xra_h, H);
    xra_r!(xra_l, L);

    fn xra_m(&mut self) -> u8 {
        self.A ^= self.read_memory(self.get_addr_hl()); 
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    fn xri(&mut self) -> u8 {
        self.A ^= self.read_8bits();
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    ora_r!(ora_a, A);
    ora_r!(ora_b, B);
    ora_r!(ora_c, C);
    ora_r!(ora_d, D);
    ora_r!(ora_e, E);
    ora_r!(ora_h, H);
    ora_r!(ora_l, L);

    fn ora_m(&mut self) -> u8 {
        self.A |= self.read_memory(self.get_addr_hl()); 
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    fn ori(&mut self) -> u8 {
        self.A |= self.read_8bits(); 
        self.set_sign((self.A | 1<<7) != 0);
        self.set_zero(self.A == 0x00);
        self.set_parity(PP8085::find_parity(self.A));
        7
    }

    cmp_r!(cmp_a, A);
    cmp_r!(cmp_b, B);
    cmp_r!(cmp_c, C);
    cmp_r!(cmp_d, D);
    cmp_r!(cmp_e, E);
    cmp_r!(cmp_h, H);
    cmp_r!(cmp_l, L);

    /// CMP M
    fn cmp_m(&mut self) -> u8 {
        match self.read_memory(self.get_addr_hl()).cmp(&self.A) {
            std::cmp::Ordering::Equal => {
                self.set_carry(false);
                self.set_zero(true);
            }
            std::cmp::Ordering::Greater => {
                self.set_carry(false);
                self.set_zero(false);
            }
            std::cmp::Ordering::Less => {
                self.set_carry(true);
                self.set_zero(false);
            }
        }
        7
    }

    /// CPI XX
    fn cpi(&mut self) -> u8 {
        match self.read_8bits().cmp(&self.A) {
            std::cmp::Ordering::Equal => {
                self.set_carry(false);
                self.set_zero(true);
            }
            std::cmp::Ordering::Greater => {
                self.set_carry(false);
                self.set_zero(false);
            }
            std::cmp::Ordering::Less => {
                self.set_carry(true);
                self.set_zero(false);
            }
        }
        4
    }

    /// JMP XXXX
    fn jmp(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.PC = ((oph as u16) << 8) | (opl as u16);
        10
    }

    /// JC XXXX
    fn jc(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if self.get_carry() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JNC XXXX
    fn jnc(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if !self.get_carry() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JZ XXXX
    fn jz(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if self.get_zero() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JNZ XXXX
    fn jnz(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if !self.get_zero() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JP XXXX
    fn jp(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if !self.get_sign() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JM XXXX
    fn jm(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if self.get_sign() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JPE XXXX
    fn jpe(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if self.get_parity() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// JPO XXXX
    fn jpo(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        if !self.get_parity() {
            self.PC = ((oph as u16) << 8) | (opl as u16);
            return 10;
        }
        7
    }

    /// PCHL
    fn pchl(&mut self) -> u8 {
        self.PC = self.get_addr_hl();
        6
    }

    /// CMA
    fn cma(&mut self) -> u8 {
        self.A = !self.A;
        4
    }

    /// STC
    fn stc(&mut self) -> u8 {
        self.set_carry(true);
        4
    }

    /// CMC
    fn cmc(&mut self) -> u8 {
        self.set_carry(!self.get_carry());
        4
    }

    /// DAA
    fn daa(&mut self) -> u8 {
        let mut num: u8 = self.A;
        if num & 0x0f > 0x09 || self.get_auxiliary_carry() {
            num += 0x06;
        } 
        if num & 0xf0 > 0x90 || self.get_carry() {
            num += 0x60;
        }
        self.A = num;
        4
    }

    /// RLC
    fn rlc(&mut self) -> u8 {
        self.set_carry(self.A & (1<<7) != 0);
        self.A = (self.A << 1) | ((self.A & (1 << 7)) >> 7);
        4
    }

    /// RRC
    fn rrc(&mut self) -> u8 {
        self.set_carry(self.A & 1 != 0);
        self.A = (self.A >> 1) | ((self.A & 1) << 7);
        4
    }

    /// RAL
    fn ral(&mut self) -> u8 {
        let n  = (self.A & (1<<7)) >> 7;
        self.A = (self.A << 1) | self.get_carry() as u8;
        self.set_carry(n == 1);
        4
    }

    /// RAR
    fn rar(&mut self) -> u8 {
        let n  = self.A & 1;
        self.A = (self.A >> 1) | (self.get_carry() as u8) << 7;
        self.set_carry(n == 1);
        4
    }

    rst_seq!(rst_0, 0x0000);
    rst_seq!(rst_1, 0x0008);
    rst_seq!(rst_2, 0x0010);
    rst_seq!(rst_3, 0x0018);
    rst_seq!(rst_4, 0x0020);
    rst_seq!(rst_5, 0x0028);
    rst_seq!(rst_6, 0x0030);
    rst_seq!(rst_7, 0x0038);

    call_seq!(call);
    call_seq!(cc, cnc, get_carry);
    call_seq!(cz, cnz, get_zero);
    call_seq!(cm, cp, get_sign);
    call_seq!(cpe, cpo, get_parity);

    ret_seq!(ret);
    ret_seq!(rz, rnz, get_zero);
    ret_seq!(rc, rnc, get_carry);
    ret_seq!(rm, rp, get_sign);
    ret_seq!(rpe, rpo, get_parity);

    // IN
    fn i_n(&mut self) -> u8 {
        let addr = self.read_8bits();
        self.A = self.read_io(addr);
        10
    }

    // OUT 
    fn out(&mut self) -> u8 {
        let addr = self.read_8bits();
        self.write_io(addr, self.A);
        10
    }

    /// RIM
    fn rim(&mut self) -> u8 {
        4
    }

    /// SIM
    fn sim(&mut self) -> u8 {
        4
    }
}

// -----------------------TESTS----------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_parity() {
        assert!(PP8085::find_parity(0x22));
        assert!(!PP8085::find_parity(0x32));
    }

    #[test]
    fn test_add_r() {
        let mut x = 0x11;
        let mut y = 0x11;
        let mut res = x+y;
        let mut cpu = PP8085::new();
        cpu.A = x;
        cpu.B = y;
        cpu.add_b();
        assert_eq!(cpu.A, res);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(cpu.get_parity());

        x = 0xff;
        y = 0x01;
        res = 0x00;
        cpu.A = x;
        cpu.L = y;
        cpu.add_l();
        assert_eq!(cpu.A, res);
        assert!(cpu.get_carry());
        assert!(cpu.get_overflow());
        assert!(cpu.get_zero());
        assert!(cpu.get_parity());

        x = 0xff;
        y = 0x02;
        res = 0x01;
        cpu.A = x;
        cpu.H = y;
        cpu.add_h();
        assert_eq!(cpu.A, res);
        assert!(cpu.get_carry());
        assert!(cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(!cpu.get_parity());

        x = 0x99;
        y = 0xdd;
        res = 0x76;
        cpu.A = x;
        cpu.D = y;
        cpu.add_d();
        assert_eq!(cpu.A, res);
        assert!(cpu.get_carry());
        assert!(cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(!cpu.get_parity());
    }

    #[test]
    fn test_adc_r() {
        let x = 0x11;
        let y = 0x11;
        let res = x+y+1;
        let mut cpu = PP8085::new();
        cpu.set_carry(true);
        cpu.A = x;
        cpu.B = y;
        cpu.adc_b();
        assert_eq!(cpu.A, res);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(!cpu.get_parity());

        let x = 0x11;
        let y = 0x11;
        let res = x+y;
        let mut cpu = PP8085::new();
        cpu.A = x;
        cpu.C = y;
        cpu.adc_c();
        assert_eq!(cpu.A, res);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(cpu.get_parity());

        let x = 0xff;
        let y = 0xdd;
        let res = 0xdd;
        let mut cpu = PP8085::new();
        cpu.set_carry(true);
        cpu.A = x;
        cpu.D = y;
        cpu.adc_d();
        assert_eq!(cpu.A, res);
        assert!(cpu.get_carry());
        assert!(cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(cpu.get_parity());
    }

    #[test]
    fn test_sub_r() {
        let x = 0x11;
        let y = 0x11;
        let res = x-y;
        let mut cpu = PP8085::new();
        cpu.A = x;
        cpu.B = y;
        cpu.sub_b();
        assert_eq!(cpu.A, res);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(cpu.get_zero());
        assert!(cpu.get_parity());

        let x = 0x01;
        let y = 0x03;
        let res = 0xfe;
        cpu.A = x;
        cpu.B = y;
        cpu.sub_b();
        assert_eq!(cpu.A, res);
        assert!(cpu.get_carry());
        assert!(cpu.get_overflow());
        assert!(!cpu.get_zero());
        assert!(!cpu.get_parity());
    }

    #[test]
    fn test_mov_rd_rs() {
        let mut cpu = PP8085::new();
        cpu.B = 0x2a;
        cpu.display();
        cpu.mov_a_b();
        assert_eq!(cpu.A, 0x2a);
        assert_eq!(cpu.B, 0x2a);
        cpu.mov_d_b();
        assert_eq!(cpu.D, 0x2a);
    }

    #[test]
    fn test_ana_r() {
        let mut cpu = PP8085::new();
        cpu.A = 0b01011100;
        cpu.B = 0b00001000;
        cpu.ana_b();
        assert_eq!(cpu.A, 0b00001000);
    }

    #[test]
    fn test_xra_r() {
        let mut cpu = PP8085::new();
        cpu.A = 0b01011100;
        cpu.B = 0b00001000;
        cpu.xra_b();
        assert_eq!(cpu.A, 0b01010100);
    }

    #[test]
    fn test_ora_r() {
        let mut cpu = PP8085::new();
        cpu.A = 0b01011100;
        cpu.B = 0b00001000;
        cpu.ora_b();
        assert_eq!(cpu.A, 0b01011100);
    }

    #[test]
    fn test_cmp_r() {
        let mut cpu = PP8085::new();
        cpu.A = 0x45;
        cpu.B = 0x55;
        cpu.cmp_b();
        assert!(!cpu.get_carry());
        assert!(!cpu.get_zero());

        cpu.A = 0x45;
        cpu.B = 0x05;
        cpu.cmp_b();
        assert!(cpu.get_carry());
        assert!(!cpu.get_zero());

        cpu.A = 0x45;
        cpu.B = 0x45;
        cpu.cmp_b();
        assert!(!cpu.get_carry());
        assert!(cpu.get_zero());
    }

    #[test]
    #[ignore]
    fn test_daa() {
        let mut cpu = PP8085::new();
        cpu.A = 0x38;
        cpu.B = 0x45;
        cpu.add_b();
        cpu.daa();
        assert_eq!(cpu.A, 0x83);
        assert!(!cpu.get_carry());
        assert!(cpu.get_sign());
        assert!(cpu.get_auxiliary_carry());
        assert!(!cpu.get_parity());
    }

    #[test]
    fn test_rlc() {
        let mut cpu = PP8085::new();
        cpu.A = 0b11001010;
        cpu.rlc();
        assert_eq!(cpu.A, 0b10010101);
        assert!(cpu.get_carry());

        cpu.A = 0b01001010;
        cpu.rlc();
        assert_eq!(cpu.A, 0b10010100);
        assert!(!cpu.get_carry());
    }

    #[test]
    fn test_rrc() {
        let mut cpu = PP8085::new();
        cpu.A = 0b11001010;
        cpu.rrc();
        assert_eq!(cpu.A, 0b01100101);
        assert!(!cpu.get_carry());

        cpu.A = 0b01001011;
        cpu.rrc();
        assert_eq!(cpu.A, 0b10100101);
        assert!(cpu.get_carry());
    }

    #[test]
    fn test_ral() {
        let mut cpu = PP8085::new();
        cpu.A = 0b11001010;
        cpu.set_carry(false);
        cpu.ral();
        assert_eq!(cpu.A, 0b10010100);
        assert!(cpu.get_carry());

        cpu.A = 0b01001011;
        cpu.set_carry(true);
        cpu.ral();
        assert_eq!(cpu.A, 0b10010111);
        assert!(!cpu.get_carry());
    }

    #[test]
    fn test_rar() {
        let mut cpu = PP8085::new();
        cpu.A = 0b11001010;
        cpu.set_carry(false);
        cpu.rar();
        assert_eq!(cpu.A, 0b01100101);
        assert!(!cpu.get_carry());

        cpu.A = 0b01001011;
        cpu.set_carry(true);
        cpu.rar();
        assert_eq!(cpu.A, 0b10100101);
        assert!(cpu.get_carry());
    }


    #[test]
    fn test_rst() {
        let mut cpu = PP8085::new();
        cpu.SP = 0x19ff;
        cpu.PC = 0x0102;
        let c = cpu.rst_1();
        assert_eq!(c, 12);
        assert_eq!(cpu.PC, 0x0008);
        assert_eq!(cpu.read_memory(cpu.SP), 0x02);
        assert_eq!(cpu.read_memory(cpu.SP+1), 0x01);
    }

    #[test]
    fn test_cond_call() {
        let mut cpu = PP8085::new();
        cpu.set_carry(true);
        cpu.SP = 0x19ff;
        cpu.PC = 0x0000;
        cpu.write_memory(0x0000, 0x22);
        cpu.write_memory(0x0001, 0xaa);
        let c = cpu.cc();
        assert_eq!(c, 18);
        assert_eq!(cpu.PC, 0xaa22);
        assert_eq!(cpu.read_memory(cpu.SP), 0x02);
        assert_eq!(cpu.read_memory(cpu.SP+1), 0x00);

        cpu.set_carry(false);
        cpu.SP = 0x19ff;
        cpu.PC = 0x0000;
        let c = cpu.cc();
        assert_eq!(c, 9);
        assert_eq!(cpu.PC, 0x0002);
    }

    #[test]
    fn test_cond_call_fn2() {
        let mut cpu = PP8085::new();
        cpu.set_carry(false);
        cpu.SP = 0x19ff;
        cpu.PC = 0x0000;
        cpu.write_memory(0x0000, 0x22);
        cpu.write_memory(0x0001, 0xaa);
        let c = cpu.cnc();
        assert_eq!(c, 18);
        assert_eq!(cpu.PC, 0xaa22);
        assert_eq!(cpu.read_memory(cpu.SP), 0x02);
        assert_eq!(cpu.read_memory(cpu.SP+1), 0x00);

        cpu.set_carry(true);
        cpu.SP = 0x19ff;
        cpu.PC = 0x0000;
        let c = cpu.cnc();
        assert_eq!(c, 9);
        assert_eq!(cpu.PC, 0x0002);
    }

    #[test]
    fn test_cond_ret() {
        let mut cpu = PP8085::new();
        cpu.set_carry(true);
        cpu.SP = 0x19fd;
        cpu.PC = 0x0000;
        cpu.write_memory(cpu.SP, 0x02);
        cpu.write_memory(cpu.SP+1, 0xaa);
        let c = cpu.rc();
        assert_eq!(c, 12);
        assert_eq!(cpu.PC, 0xaa02);
        assert_eq!(cpu.SP, 0x19ff);

        cpu.set_carry(false);
        cpu.SP = 0x19fd;
        cpu.PC = 0x0000;
        let c = cpu.rc();
        assert_eq!(c, 6);
        assert_eq!(cpu.PC, 0x0000);
        assert_eq!(cpu.SP, 0x19fd);
    }

    #[test]
    fn test_cond_ret_fn2() {
        let mut cpu = PP8085::new();
        cpu.set_carry(false);
        cpu.SP = 0x2000-2;
        cpu.PC = 0x0000;
        cpu.write_memory(cpu.SP, 0x02);
        cpu.write_memory(cpu.SP+1, 0x12);
        let c = cpu.rnc();
        assert_eq!(c, 12);
        assert_eq!(cpu.PC, 0x1202);
        assert_eq!(cpu.SP, 0x2000);

        cpu.set_carry(true);
        cpu.SP = 0x19fd;
        cpu.PC = 0x0000;
        let c = cpu.rnc();
        assert_eq!(c, 6);
        assert_eq!(cpu.PC, 0x0000);
        assert_eq!(cpu.SP, 0x19fd);
    }

    #[test]
    fn test_dad() {
        let mut cpu = PP8085::new();
        cpu.H = 0x01;
        cpu.L = 0x02;
        cpu.B = 0x03;
        cpu.C = 0x04;
        cpu.dad_b();
        assert_eq!(cpu.get_addr_hl(), 0x0406);
        assert!(!cpu.get_carry());

        cpu.H = 0x00;
        cpu.L = 0x02;
        cpu.B = 0xff;
        cpu.C = 0xff;
        cpu.dad_b();
        assert_eq!(cpu.get_addr_hl(), 0x0001);
        assert!(cpu.get_carry());
    }

    #[test]
    fn test_io_connections() {
        let mut cpu = PP8085::new();
        cpu.add_io_port(0x05);
        cpu.write_io(0x05, 0xaf);
        assert_eq!(cpu.read_io(0x05), 0xaf);

        cpu.remove_io_port(0x05);
        cpu.write_io(0x05, 0xaf);
        assert_eq!(cpu.read_io(0x05), 0);
    }

    #[test]
    fn test_in() {
        let mut cpu = PP8085::new();
        cpu.add_io_port(0x05);
        cpu.write_io(0x05, 0xaf);
        cpu.PC = 0x000a;
        cpu.write_memory(0x000a, 0x05);
        cpu.i_n();
        assert_eq!(cpu.A, 0xaf);
    }

    #[test]
    fn test_out() {
        let mut cpu = PP8085::new();
        cpu.add_io_port(0x05);
        cpu.PC = 0x000a;
        cpu.write_memory(0x000a, 0x05);
        cpu.A = 0xaf;
        cpu.out();
        assert_eq!(cpu.read_io(0x05), 0xaf);
    }

    #[test]
    fn test_inr() {
        let mut cpu = PP8085::new();
        cpu.B = 0x00;
        for x in 0..0xff {
            assert_eq!(x, cpu.B);
            cpu.inr_b();
        }
        assert_eq!(cpu.B, 0xff);
        cpu.inr_b();
        assert_eq!(cpu.B, 0x00);
        assert!(cpu.get_overflow());
    }

    #[test]
    fn test_dcr() {
        let mut cpu = PP8085::new();
        cpu.B = 0xfe;
        for x in (0x00..0xff).rev() {
            assert_eq!(cpu.B, x);
            cpu.dcr_b();
        }
        assert_eq!(cpu.B, 0xff);
        assert!(cpu.get_overflow());
    }

    #[test]
    fn test_memory_run() {
        let mut cpu = PP8085::new();
        let mut rom = Memory::new(8192);
        rom.write(0x0000, 0x3e); // mvi a
        rom.write(0x0001, 0x44);
        rom.write(0x0002, 0x16); // mvi d
        rom.write(0x0003, 0x32);
        rom.write(0x0004, 0x92); // sub d
        rom.write(0x0005, 0xd3); // out
        rom.write(0x0006, 0xdf); 
        rom.write(0x0007, 0x76); // hlt
        cpu.load_memory(rom);
        cpu.add_io_port(0xdf);
        cpu.run();
        cpu.display();
        println!("{:#02x}", cpu.read_io(0xdf));
    }
}
