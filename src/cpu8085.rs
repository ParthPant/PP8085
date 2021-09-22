use super::memory::Memory;
use super::ioport::IoPort;
use std::collections::HashMap;

#[allow(non_snake_case)]
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

    PC:u16,// Program Counter Register
    SP:u16,// Stack Pointer

    memory: Box<Memory>,
    io_ports: HashMap<u8, Box<IoPort>>,

    cycles: u32,
    IE: bool, // Interrupt enable
    HLT: bool // indicates hlt state
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
                self.set_auxiliary_carry((((num+1) & 0x0f) - 0x01) & 0x10 == 0x10);
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
    /// creates a new cpu and initializes everything to zero.
    pub fn new() -> PP8085 {
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

            memory: Box::new(Memory::new(8192)),
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
                let _ins = self.read_8bits();
                self.PC += 1;
                // decode opcode;
                // execute relevant function
            }
            self.cycles -= 1;
        }
    }

    /// display the contents of all the registers.
    pub fn display(&self) {
        println!("A : {:#02x}       F : {:#02x}", self.A, self.F);
        println!("B : {:#02x}       C : {:#02x}", self.B, self.C);
        println!("D : {:#02x}       E : {:#02x}", self.D, self.E);
        println!("H : {:#02x}       L : {:#02x}", self.H, self.L);
        println!("PC: {:#04x}", self.PC);
        println!("SP: {:#04x}", self.SP);
        println!("-----------------------------");
    }

    pub fn add_io_port(&mut self, addr: u8) {
        self.io_ports.insert(addr, Box::new(IoPort::new(addr)));
    }

    pub fn remove_io_port(&mut self, addr: u8) {
        self.io_ports.remove(&addr);
    }

    fn read_io(&mut self, addr: u8) -> u8 {
        match self.io_ports.get(&addr) {
            Some(d) => d.read(),
            None => 0,
        }
    }

    fn write_io(&mut self, addr: u8, data: u8) {
        if let Some(port) = self.io_ports.get_mut(&addr) {
            port.write(data);
        };
    }

    fn write_memory(&mut self, addr: u16, content: u8) {
        self.memory.write(addr, content);
    }

    fn read_memory(&mut self, addr: u16) -> u8 {
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
        self.L = self.read_memory(self.SP); self.SP += 1; self.H = self.read_memory(self.SP); self.SP += 1;
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
        assert_eq!(cpu.memory.read(cpu.SP), 0x02);
        assert_eq!(cpu.memory.read(cpu.SP+1), 0x01);
    }

    #[test]
    fn test_cond_call() {
        let mut cpu = PP8085::new();
        cpu.set_carry(true);
        cpu.SP = 0x19ff;
        cpu.PC = 0x0000;
        cpu.memory.write(0x0000, 0x22);
        cpu.memory.write(0x0001, 0xaa);
        let c = cpu.cc();
        assert_eq!(c, 18);
        assert_eq!(cpu.PC, 0xaa22);
        assert_eq!(cpu.memory.read(cpu.SP), 0x02);
        assert_eq!(cpu.memory.read(cpu.SP+1), 0x00);

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
        cpu.memory.write(0x0000, 0x22);
        cpu.memory.write(0x0001, 0xaa);
        let c = cpu.cnc();
        assert_eq!(c, 18);
        assert_eq!(cpu.PC, 0xaa22);
        assert_eq!(cpu.memory.read(cpu.SP), 0x02);
        assert_eq!(cpu.memory.read(cpu.SP+1), 0x00);

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
        cpu.memory.write(cpu.SP, 0x02);
        cpu.memory.write(cpu.SP+1, 0xaa);
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
        cpu.memory.write(cpu.SP, 0x02);
        cpu.memory.write(cpu.SP+1, 0x12);
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

    fn test_inr() {
        let mut cpu = PP8085::new();
        cpu.B = 0x00;
        for x in 0..0xff {
            assert_eq!(x, cpu.B);
            cpu.inr_b();
        }
        assert_eq!(cpu.B, 0);
        assert!(cpu.get_overflow());
    }

    fn test_dcr() {
        let mut cpu = PP8085::new();
        cpu.B = 0xff;
        for x in 0xff..0x00 {
            assert_eq!(cpu.B, x);
            cpu.dcr_b();
        }
        assert_eq!(cpu.B, 0xff);
        assert!(cpu.get_overflow());
    }
}