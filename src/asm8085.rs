use std::fs::File;
use std::i64;
use std::io::{Result, Read};
use std::result::Result as Res;
use std::collections::HashMap;
use std::fmt;
use wasm_bindgen::prelude::*;

enum Token {
    Mnemonic(String, usize, usize), // main instruction mnemonic
    Operand(String),          // operant that is the part of the instruction
    Data(i16),                // 8 or 16 bit data
    Label(String),            // line label
    Symbol(String),           // symbols
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Mnemonic(s, n_r, n_o) => write!(f, "Mnemonic({},{},{})", s, n_r, n_o),
            Operand(s)            => write!(f, "Operand({})", s),
            Data(i)               => write!(f, "Data({})", i),
            Label(s)              => write!(f, "Label({})", s),
            Symbol(s)             => write!(f, "Symbol({})", s),
        }
    }
}

fn read_file(filename: &str) -> Result<String> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

fn get_words(data: &str) -> Vec<Vec<&str>> {
    let mut res: Vec<Vec<&str>> = Vec::new();
    for line in data.lines() {
        let mut l: Vec<&str> = Vec::new();
        for word in line.split_whitespace() {
            match word {
                ";" => break,
                "," => break, // no need for this here
                " " => break, 
                _ => {
                    if word.chars().next().unwrap() == ';' { // ;Comments :')
                        break;
                    }
                    l.push(word)
                },
            }
        }
        if l.len() > 0 {
            res.push(l);
        }
    }
    res
}

fn lex_line<'a>(line: &'a Vec<&str>) -> Res<Vec<Token>, &'a str> {
    let mut i = 0;
    let mut res: Vec<Token> = Vec::new();
    while i < line.len() {
        if let Some(t) = tokenize(line[i]) {
            res.push(t);
        }
        i += 1;
    }
    Ok(res)
}

pub fn assemble(code: &str) -> (Vec<u8>, String) {
    let code = code.replace(",", " ");
    let parsed = get_words(&code);

    let mut tokens: Vec<Token> = Vec::new();
    for line in parsed.iter() {
        match lex_line(line) {
            Ok(mut l) => tokens.append(&mut l),
            Err(e) => panic!("{}",e),
        }
    }

    // First Pass: build the symbol table
    let mut symbol_table: HashMap<String, usize> = HashMap::new();
    let mut addr = 0;
    for token in tokens.iter() {
        match token {
            Token::Mnemonic(_, _, n_o)   => {addr += n_o + 1;},
            Token::Label(s)              => {symbol_table.insert(s.clone(), addr);},
            _                            => ()
        };
    };

    // Second Pass: generate the rom
    let mut bin: Vec<u8> = Vec::new();
    let mut listing = String::new();
    let mut addr: usize = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Mnemonic(s, n_r, n_o) => {
                let mut ins: String = s.clone();
                let mut num_registers: usize = n_r.clone();
                let mut loc = i+1;
                while num_registers > 0 {
                    if let Token::Operand(r) = &tokens[loc] {
                        ins.push('_');
                        ins.push_str(r);
                    } else {
                        panic!("{}: cannot complete instruction {}", i, ins)
                    }
                    num_registers -= 1;
                    loc += 1;
                }
                bin.push(get_opcode(&ins));
                listing.push_str(&format!("{:#06x}\t{}", addr, ins));

                let num_bytes = *n_o;
                let mut val: u16 = 0;

                if num_bytes > 0 {
                    if let Token::Data(d) = &tokens[loc] {
                        val = d.clone() as u16;
                    } else if let Token::Symbol(s) = &tokens[loc] {
                        val = symbol_table.get(s).unwrap().clone() as u16;
                    }

                    bin.push((val & 0x00ff) as u8);
                    if num_bytes == 2 {
                        bin.push((val >> 8) as u8);
                        listing.push_str(&format!(" {:#06x}", val));
                    } else {
                        listing.push_str(&format!(" {:#02x}", val));
                    }
                }

                addr += num_bytes + 1;
                listing.push_str("\n");
            },
            _  => ()
        };
    };

    (bin, listing)
}

pub fn parse(filename: &str) -> (Vec<u8>, String) {
    let file = read_file(filename).unwrap();
    assemble(&file)
}

#[wasm_bindgen]
pub fn parse_wasm(data: &str) -> Vec<u8> {
    assemble(&data).0
}

fn get_opcode(ins: &str) -> u8 {
    match ins {
        "nop"      => 0x00,
        "lxi_b"    => 0x01,
        "stax_b"   => 0x02,
        "inx_b"    => 0x03,
        "inr_b"    => 0x04,
        "dcr_b"    => 0x05,
        "mvi_b"    => 0x06,
        "rlc"      => 0x07,
        "dad_b"    => 0x09,
        "ldax_b"   => 0x0A,
        "dcx_b"    => 0x0B,
        "inr_c"    => 0x0C,
        "dcr_c"    => 0x0D,
        "mvi_c"    => 0x0E,
        "rrc"      => 0x0F,
        "lxi_d"    => 0x11,
        "stax_d"   => 0x12,
        "inx_d"    => 0x13,
        "inr_d"    => 0x14,
        "dcr_d"    => 0x15,
        "mvi_d"    => 0x16,
        "ral"      => 0x17,
        "dad_d"    => 0x19,
        "ldax_d"   => 0x1A,
        "dcx_d"    => 0x1B,
        "inr_e"    => 0x1C,
        "dcr_e"    => 0x1D,
        "mvi_e"    => 0x1E,
        "rar"      => 0x1F,
        "rim"      => 0x20,
        "lxi_h"    => 0x21,
        "shld"     => 0x22,
        "inx_h"    => 0x23,
        "inr_h"    => 0x24,
        "dcr_h"    => 0x25,
        "mvi_h"    => 0x26,
        "daa"      => 0x27,
        "dad_h"    => 0x29,
        "lhld"     => 0x2A,
        "dcx_h"    => 0x2B,
        "inr_l"    => 0x2C,
        "dcr_l"    => 0x2D,
        "mvi_l"    => 0x2E,
        "cma"      => 0x2F,
        "sim"      => 0x30,
        "lxi_sp"   => 0x31,
        "sta"      => 0x32,
        "inx_sp"   => 0x33,
        "inr_m"    => 0x34,
        "dcr_m"    => 0x35,
        "mvi_m"    => 0x36,
        "stc"      => 0x37,
        "dad_sp"   => 0x39,
        "lda"      => 0x3A,
        "dcx_sp"   => 0x3B,
        "inr_a"    => 0x3C,
        "dcr_a"    => 0x3D,
        "mvi_a"    => 0x3E,
        "cmc"      => 0x3F,
        "mov_b_b"  => 0x40,
        "mov_b_c"  => 0x41,
        "mov_b_d"  => 0x42,
        "mov_b_e"  => 0x43,
        "mov_b_h"  => 0x44,
        "mov_b_l"  => 0x45,
        "mov_b_m"  => 0x46,
        "mov_b_a"  => 0x47,
        "mov_c_b"  => 0x48,
        "mov_c_c"  => 0x49,
        "mov_c_d"  => 0x4A,
        "mov_c_e"  => 0x4B,
        "mov_c_h"  => 0x4C,
        "mov_c_l"  => 0x4D,
        "mov_c_m"  => 0x4E,
        "mov_c_a"  => 0x4F,
        "mov_d_b"  => 0x50,
        "mov_d_c"  => 0x51,
        "mov_d_d"  => 0x52,
        "mov_d_e"  => 0x53,
        "mov_d_h"  => 0x54,
        "mov_d_l"  => 0x55,
        "mov_d_m"  => 0x56,
        "mov_d_a"  => 0x57,
        "mov_e_b"  => 0x58,
        "mov_e_c"  => 0x59,
        "mov_e_d"  => 0x5A,
        "mov_e_e"  => 0x5B,
        "mov_e_h"  => 0x5C,
        "mov_e_l"  => 0x5D,
        "mov_e_m"  => 0x5E,
        "mov_e_a"  => 0x5F,
        "mov_h_b"  => 0x60,
        "mov_h_c"  => 0x61,
        "mov_h_d"  => 0x62,
        "mov_h_e"  => 0x63,
        "mov_h_h"  => 0x64,
        "mov_h_l"  => 0x65,
        "mov_h_m"  => 0x66,
        "mov_h_a"  => 0x67,
        "mov_l_b"  => 0x68,
        "mov_l_c"  => 0x69,
        "mov_l_d"  => 0x6A,
        "mov_l_e"  => 0x6B,
        "mov_l_h"  => 0x6C,
        "mov_l_l"  => 0x6D,
        "mov_l_m"  => 0x6E,
        "mov_l_a"  => 0x6F,
        "mov_m_b"  => 0x70,
        "mov_m_c"  => 0x71,
        "mov_m_d"  => 0x72,
        "mov_m_e"  => 0x73,
        "mov_m_h"  => 0x74,
        "mov_m_l"  => 0x75,
        "hlt"      => 0x76,
        "mov_m_a"  => 0x77,
        "mov_a_b"  => 0x78,
        "mov_a_c"  => 0x79,
        "mov_a_d"  => 0x7A,
        "mov_a_e"  => 0x7B,
        "mov_a_h"  => 0x7C,
        "mov_a_l"  => 0x7D,
        "mov_a_m"  => 0x7E,
        "mov_a_a"  => 0x7F,
        "add_b"    => 0x80,
        "add_c"    => 0x81,
        "add_d"    => 0x82,
        "add_e"    => 0x83,
        "add_h"    => 0x84,
        "add_l"    => 0x85,
        "add_m"    => 0x86,
        "add_a"    => 0x87,
        "adc_b"    => 0x88,
        "adc_c"    => 0x89,
        "adc_d"    => 0x8A,
        "adc_e"    => 0x8B,
        "adc_h"    => 0x8C,
        "adc_l"    => 0x8D,
        "adc_m"    => 0x8E,
        "adc_a"    => 0x8F,
        "sub_b"    => 0x90,
        "sub_c"    => 0x91,
        "sub_d"    => 0x92,
        "sub_e"    => 0x93,
        "sub_h"    => 0x94,
        "sub_l"    => 0x95,
        "sub_m"    => 0x96,
        "sub_a"    => 0x97,
        "sbb_b"    => 0x98,
        "sbb_c"    => 0x99,
        "sbb_d"    => 0x9A,
        "sbb_e"    => 0x9B,
        "sbb_h"    => 0x9C,
        "sbb_l"    => 0x9D,
        "sbb_m"    => 0x9E,
        "sbb_a"    => 0x9F,
        "ana_b"    => 0xA0,
        "ana_c"    => 0xA1,
        "ana_d"    => 0xA2,
        "ana_e"    => 0xA3,
        "ana_h"    => 0xA4,
        "ana_l"    => 0xA5,
        "ana_m"    => 0xA6,
        "ana_a"    => 0xA7,
        "xra_b"    => 0xA8,
        "xra_c"    => 0xA9,
        "xra_d"    => 0xAA,
        "xra_e"    => 0xAB,
        "xra_h"    => 0xAC,
        "xra_l"    => 0xAD,
        "xra_m"    => 0xAE,
        "xra_a"    => 0xAF,
        "ora_b"    => 0xB0,
        "ora_c"    => 0xB1,
        "ora_d"    => 0xB2,
        "ora_e"    => 0xB3,
        "ora_h"    => 0xB4,
        "ora_l"    => 0xB5,
        "ora_m"    => 0xB6,
        "ora_a"    => 0xB7,
        "cmp_b"    => 0xB8,
        "cmp_c"    => 0xB9,
        "cmp_d"    => 0xBA,
        "cmp_e"    => 0xBB,
        "cmp_h"    => 0xBC,
        "cmp_l"    => 0xBD,
        "cmp_m"    => 0xBE,
        "cmp_a"    => 0xBF,
        "rnz"      => 0xC0,
        "pop_b"    => 0xC1,
        "jnz"      => 0xC2,
        "jmp"      => 0xC3,
        "cnz"      => 0xC4,
        "push_b"   => 0xC5,
        "adi"      => 0xC6,
        "rst_0"    => 0xC7,
        "rz"       => 0xC8,
        "ret"      => 0xC9,
        "jz"       => 0xCA,
        "cz"       => 0xCC,
        "call"     => 0xCD,
        "aci"      => 0xCE,
        "rst_1"    => 0xCF,
        "rnc"      => 0xD0,
        "pop_d"    => 0xD1,
        "jnc"      => 0xD2,
        "out"      => 0xD3,
        "cnc"      => 0xD4,
        "push_d"   => 0xD5,
        "sui"      => 0xD6,
        "rst_2"    => 0xD7,
        "rc"       => 0xD8,
        "jc"       => 0xDA,
        "i_n"      => 0xDB,
        "cc"       => 0xDC,
        "sbi"      => 0xDE,
        "rst_3"    => 0xDF,
        "rpo"      => 0xE0,
        "pop_h"    => 0xE1,
        "jpo"      => 0xE2,
        "xthl"     => 0xE3,
        "cpo"      => 0xE4,
        "push_h"   => 0xE5,
        "ani"      => 0xE6,
        "rst_4"    => 0xE7,
        "rpe"      => 0xE8,
        "pchl"     => 0xE9,
        "jpe"      => 0xEA,
        "xchg"     => 0xEB,
        "cpe"      => 0xEC,
        "xri"      => 0xEE,
        "rst_5"    => 0xEF,
        "rp"       => 0xF0,
        "pop_psw"  => 0xF1,
        "jp"       => 0xF2,
        "di"       => 0xF3,
        "cp"       => 0xF4,
        "push_psw" => 0xF5,
        "ori"      => 0xF6,
        "rst_6"    => 0xF7,
        "rm"       => 0xF8,
        "sphl"     => 0xF9,
        "jm"       => 0xFA,
        "ei"       => 0xFB,
        "cm"       => 0xFC,
        "cpi"      => 0xFE,
        "rst_7"    => 0xFF,
        _          => {panic!("{} not found", ins)}
    }
}

fn tokenize(word: &str) -> Option<Token> {
    let word = word.to_lowercase();
    if word.contains(':') {
        let temp = word.trim_end_matches(':').to_string();
        return Some(Token::Label(temp));
    }
    let data = word.clone();
    let t = match data.as_ref() {
        //                             Ins   nR nO
        "aci"  => Some(Token::Mnemonic(data, 0, 1)),   
        "adc"  => Some(Token::Mnemonic(data, 1, 0)),   
        "add"  => Some(Token::Mnemonic(data, 1, 0)),   
        "adi"  => Some(Token::Mnemonic(data, 0, 1)),   
        "ana"  => Some(Token::Mnemonic(data, 1, 0)),   
        "ani"  => Some(Token::Mnemonic(data, 0, 1)),   
        "call" => Some(Token::Mnemonic(data, 0, 2)),   
        "cc"   => Some(Token::Mnemonic(data, 0, 2)),   
        "cm"   => Some(Token::Mnemonic(data, 0, 2)),   
        "cma"  => Some(Token::Mnemonic(data, 0, 0)),   
        "cmc"  => Some(Token::Mnemonic(data, 0, 0)),   
        "cmp"  => Some(Token::Mnemonic(data, 1, 2)),   
        "cnc"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cnz"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cp"   => Some(Token::Mnemonic(data, 0, 2)),   
        "cpe"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cpi"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cpo"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cz"   => Some(Token::Mnemonic(data, 0, 2)),   
        "daa"  => Some(Token::Mnemonic(data, 0, 0)),   
        "dcr"  => Some(Token::Mnemonic(data, 1, 0)),   
        "dcx"  => Some(Token::Mnemonic(data, 1, 0)),   
        "di"   => Some(Token::Mnemonic(data, 0, 0)),   
        "ei"   => Some(Token::Mnemonic(data, 0, 0)),   
        "hlt"  => Some(Token::Mnemonic(data, 0, 0)),   
        "in"   => Some(Token::Mnemonic(data, 0, 1)),   
        "inr"  => Some(Token::Mnemonic(data, 1, 0)),   
        "inx"  => Some(Token::Mnemonic(data, 1, 0)),   
        "jc"   => Some(Token::Mnemonic(data, 0, 2)),   
        "jm"   => Some(Token::Mnemonic(data, 0, 2)),   
        "jmp"  => Some(Token::Mnemonic(data, 0, 2)),   
        "jnc"  => Some(Token::Mnemonic(data, 0, 2)),   
        "jnz"  => Some(Token::Mnemonic(data, 0, 2)),   
        "jp"   => Some(Token::Mnemonic(data, 0, 2)),   
        "jpe"  => Some(Token::Mnemonic(data, 0, 2)),   
        "jpo"  => Some(Token::Mnemonic(data, 0, 2)),   
        "jz"   => Some(Token::Mnemonic(data, 0, 2)),   
        "lda"  => Some(Token::Mnemonic(data, 0, 2)),   
        "ldax" => Some(Token::Mnemonic(data, 1, 0)),   
        "lhld" => Some(Token::Mnemonic(data, 0, 2)),   
        "lxi"  => Some(Token::Mnemonic(data, 1, 2)),   
        "mov"  => Some(Token::Mnemonic(data, 2, 0)),   
        "mvi"  => Some(Token::Mnemonic(data, 1, 1)),   
        "nop"  => Some(Token::Mnemonic(data, 0, 0)),   
        "ora"  => Some(Token::Mnemonic(data, 1, 0)),   
        "ori"  => Some(Token::Mnemonic(data, 0, 1)),   
        "out"  => Some(Token::Mnemonic(data, 0, 1)),   
        "pchl" => Some(Token::Mnemonic(data, 0, 0)),   
        "pop"  => Some(Token::Mnemonic(data, 1, 0)),   
        "push" => Some(Token::Mnemonic(data, 1, 0)),   
        "ral"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rar"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rc"   => Some(Token::Mnemonic(data, 0, 0)),   
        "ret"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rim"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rlc"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rm"   => Some(Token::Mnemonic(data, 0, 0)),   
        "rnc"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rnz"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rp"   => Some(Token::Mnemonic(data, 0, 0)),   
        "rpe"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rpo"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rrc"  => Some(Token::Mnemonic(data, 0, 0)),   
        "rst"  => Some(Token::Mnemonic(data, 1, 0)),   
        "rz"   => Some(Token::Mnemonic(data, 0, 0)),   
        "sbb"  => Some(Token::Mnemonic(data, 1, 0)),   
        "sbi"  => Some(Token::Mnemonic(data, 0, 1)),   
        "shld" => Some(Token::Mnemonic(data, 0, 2)),   
        "sim"  => Some(Token::Mnemonic(data, 0, 0)),   
        "sphl" => Some(Token::Mnemonic(data, 0, 0)),   
        "sta"  => Some(Token::Mnemonic(data, 0, 2)),   
        "stax" => Some(Token::Mnemonic(data, 1, 0)),   
        "stc"  => Some(Token::Mnemonic(data, 0, 0)),   
        "sub"  => Some(Token::Mnemonic(data, 1, 0)),   
        "sui"  => Some(Token::Mnemonic(data, 0, 1)),   
        "xchg" => Some(Token::Mnemonic(data, 0, 0)),   
        "xra"  => Some(Token::Mnemonic(data, 1, 0)),   
        "xri"  => Some(Token::Mnemonic(data, 0, 1)),   
        "xthl" => Some(Token::Mnemonic(data, 0, 0)),   
        "a"    => Some(Token::Operand(data)),
        "b"    => Some(Token::Operand(data)),
        "c"    => Some(Token::Operand(data)),
        "d"    => Some(Token::Operand(data)),
        "e"    => Some(Token::Operand(data)),
        "h"    => Some(Token::Operand(data)),
        "l"    => Some(Token::Operand(data)),
        "m"    => Some(Token::Operand(data)),
        "ab"   => Some(Token::Operand(data)),
        "bc"   => Some(Token::Operand(data)),
        "de"   => Some(Token::Operand(data)),
        "hl"   => Some(Token::Operand(data)),
        "psw"  => Some(Token::Operand(data)),
        "sp"   => Some(Token::Operand(data)),
        "0"    => Some(Token::Operand(data)),
        "1"    => Some(Token::Operand(data)),
        "2"    => Some(Token::Operand(data)),
        "3"    => Some(Token::Operand(data)),
        "4"    => Some(Token::Operand(data)),
        "5"    => Some(Token::Operand(data)),
        "6"    => Some(Token::Operand(data)),
        "7"    => Some(Token::Operand(data)),
        _      => Some(Token::Symbol(data)),
    };
    if word.contains('h') {
        let word = word.trim_end_matches('h');
        if let Ok(word) = i64::from_str_radix(word, 16) {
            return Some(Token::Data(word as i16));
        }
    }
    t
}
