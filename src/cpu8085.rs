use super::memory::Memory;

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
            self.memory.write(self.get_addr_hl(), self.$source);
            7
        }
    };
}

macro_rules!  mov_rd_m {
    ($fn_name: ident, $dest: ident) => {
        fn $fn_name (&mut self) -> u8 {
            self.$dest = self.memory.read(self.get_addr_hl());
            7
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

    memory: Memory,
    cycles: u32,
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

            memory: Memory::new(8192),
            cycles: 0,
        }
    }

    /// execution cycle
    pub fn run(&mut self) {
        loop {
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
        let r = self.memory.read(self.PC);
        self.PC += 1;
        r
    }

    fn read_16bits(&mut self) ->  (u8, u8) {
        let l = self.memory.read(self.PC);
        self.PC += 1;
        let h = self.memory.read(self.PC);
        self.PC += 1;
        (l, h)
    }

    /// NOP
    fn nop(&mut self) -> u8 {
        4
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
        self.memory.write(self.get_addr_hl(), op);
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
        self.memory.write(self.get_addr_bc(), self.A);
        7
    }

    /// STAX D
    /// store A indirect
    fn stax_d(&mut self) -> u8 {
        self.memory.write(self.get_addr_de(), self.A);
        7
    }

    /// LDAX B
    /// load A indirect
    fn ldax_b(&mut self) -> u8 {
        self.A = self.memory.read(self.get_addr_bc());
        7
    }
    
    /// LDAX D
    /// load A indirect
    fn ldax_d(&mut self) -> u8 {
        self.A = self.memory.read(self.get_addr_de());
        7
    }

    /// STA XXXX
    /// store A direct
    fn sta(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.memory.write((oph as u16) << 8 | opl as u16, self.A);
        13 
    }
    
    /// LDA XXXX
    /// load A direct
    fn lda(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        self.A = self.memory.read((oph as u16) << 8 | opl as u16);
        13 
    }

    /// SHLD XXXX
    /// store HL direct
    fn shld(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        let addr = (oph as u16) << 8 | opl as u16;
        self.memory.write(addr, self.H);
        self.memory.write(addr+1, self.L);
        16 
    }
    
    /// LHLD XXXX
    /// load HL direct
    fn lhld(&mut self) -> u8 {
        let (opl, oph) = self.read_16bits();
        let addr = (oph as u16) << 8 | opl as u16;
        self.L = self.memory.read(addr);
        self.H = self.memory.read(addr+1);
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
        self.memory.write(self.SP, self.B);
        self.SP -= 1;
        self.memory.write(self.SP, self.C);
        0
    }
    
    /// PUSH D
    /// push DE on stack
    fn push_d(&mut self) -> u8 {
        self.SP -= 1;
        self.memory.write(self.SP, self.D);
        self.SP -= 1;
        self.memory.write(self.SP, self.E);
        12
    }

    /// PUSH H
    /// push HL on stack
    fn push_h(&mut self) -> u8 {
        self.SP -= 1;
        self.memory.write(self.SP, self.H);
        self.SP -= 1;
        self.memory.write(self.SP, self.L);
        12
    }

    /// PUSH PSW 
    /// push A and F on stack
    fn push_psw(&mut self) -> u8 {
        self.SP -= 1;
        self.memory.write(self.SP, self.A);
        self.SP -= 1;
        self.memory.write(self.SP, self.F);
        12
    }

    /// POP B
    /// pop BC from stack
    fn pop_b(&mut self) -> u8 {
        self.C = self.memory.read(self.SP);
        self.SP += 1;
        self.B = self.memory.read(self.SP);
        self.SP += 1;
        10
    }

    /// POP D
    /// pop DE from stack
    fn pop_d(&mut self) -> u8 {
        self.E = self.memory.read(self.SP);
        self.SP += 1;
        self.D = self.memory.read(self.SP);
        self.SP += 1;
        10
    }

    /// POP H
    /// pop HL from stack
    fn pop_h(&mut self) -> u8 {
        self.L = self.memory.read(self.SP); self.SP += 1; self.H = self.memory.read(self.SP); self.SP += 1;
        10
    }

    /// POP PSW
    /// pop A and F from stack
    fn pop_psw(&mut self) -> u8 {
        self.F = self.memory.read(self.SP);
        self.SP += 1;
        self.A = self.memory.read(self.SP);
        self.SP += 1;
        10
    }

    /// XTHL
    /// exchange stack with HL
    fn xthl(&mut self) -> u8 {
        let top = self.memory.read(self.SP);
        let bottom = self.memory.read(self.SP+1);

        self.memory.write(self.SP, self.L);
        self.memory.write(self.SP+1, self.H);

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

    /// INR B
    /// inrement B by 1
    fn inr_b(&mut self) -> u8 {
        if self.B < 0xff {
            self.B += 1;
        } else {
            self.B = 0x00;
        }
        let num = self.B;
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

    /// INR C
    /// inrement C by 1
    fn inr_c(&mut self) -> u8 {
        if self.C < 0xff {
            self.C += 1;
        } else {
            self.C = 0x00;
        }
        let num = self.C;
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

    /// INR D
    /// inrement D by 1
    fn inr_d(&mut self) -> u8 {
        if self.D < 0xff {
            self.D += 1;
        } else {
            self.D = 0x00;
        }
        let num = self.D;
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

    /// INR E
    /// inrement E by 1
    fn inr_e(&mut self) -> u8 {
        if self.E < 0xff {
            self.E += 1;
        } else {
            self.E = 0x00;
        }
        let num = self.E;
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

    /// INR H
    /// inrement H by 1
    fn inr_h(&mut self) -> u8 {
        if self.H < 0xff {
            self.H += 1;
        } else {
            self.H = 0x00;
        }
        let num = self.H;
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

    /// INR L
    /// inrement L by 1
    fn inr_l(&mut self) -> u8 {
        if self.L < 0xff {
            self.L += 1;
        } else {
            self.L = 0x00;
        }
        let num = self.L;
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
    
    /// INR A
    /// inrement A by 1
    fn inr_a(&mut self) -> u8 {
        if self.A < 0xff {
            self.A += 1;
        } else {
            self.A = 0x00;
        }
        let num = self.A;
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

    /// INR M
    /// inrement M by 1
    fn inr_m(&mut self) -> u8 {
        let mut num = self.memory.read(self.get_addr_hl());
        if num < 0xff {
            num += 1;
            self.memory.write(self.get_addr_hl(), num);
        } else {
            num = 0x00;
            self.memory.write(self.get_addr_hl(), num);
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

    /// DCR B
    /// decrement B by 1
    fn dcr_b(&mut self) -> u8 {
        if self.B > 0x00 {
            self.B -= 1;
        } else {
            self.B = 0xff;
        }
        let num = self.B;
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

    /// DCR C
    /// decrement C by 1
    fn dcr_c(&mut self) -> u8 {
        if self.C > 0x00 {
            self.C -= 1;
        } else {
            self.C = 0xff;
        }
        let num = self.C;
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

    /// DCR D
    /// decrement D by 1
    fn dcr_d(&mut self) -> u8 {
        if self.D > 0x00 {
            self.D -= 1;
        } else {
            self.D = 0xff;
        }
        let num = self.D;
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

    /// DCR E
    /// decrement E by 1
    fn dcr_e(&mut self) -> u8 {
        if self.E > 0x00 {
            self.E -= 1;
        } else {
            self.E = 0xff;
        }
        let num = self.E;
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

    /// DCR H
    /// decrement H by 1
    fn dcr_h(&mut self) -> u8 {
        if self.H > 0x00 {
            self.H -= 1;
        } else {
            self.H = 0xff;
        }
        let num = self.H;
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

    /// DCR L
    /// decrement L by 1
    fn dcr_l(&mut self) -> u8 {
        if self.L > 0x00 {
            self.L -= 1;
        } else {
            self.L = 0xff;
        }
        let num = self.L;
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

    /// DCR A
    /// decrement A by 1
    fn dcr_a(&mut self) -> u8 {
        if self.A > 0x00 {
            self.A -= 1;
        } else {
            self.A = 0xff;
        }
        let num = self.A;
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

    /// DCR M
    /// decrement M by 1
    fn dcr_m(&mut self) -> u8 {
        let mut num = self.memory.read(self.get_addr_hl());
        if num > 0x00 {
            num -= 1;
            self.memory.write(self.get_addr_hl(), num);
        } else {
            num = 0xff;
            self.memory.write(self.get_addr_hl(), num);
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
        let num = self.memory.read(self.get_addr_hl());
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
        let num = self.memory.read(self.get_addr_hl()) + (self.F & 1);
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
        let num = self.memory.read(self.get_addr_hl());
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
        let num = self.memory.read(self.get_addr_hl()) + (self.F & 1);
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
}