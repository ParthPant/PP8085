use std::fs::File;
use std::i64;
use std::io::{Result, Read};
use std::result::Result as Res;
use std::collections::HashMap;
use std::fmt;

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
        "cma"  => Some(Token::Mnemonic(data, 0, 2)),   
        "cmc"  => Some(Token::Mnemonic(data, 0, 2)),   
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

fn read_file(filename: &str) -> Result<String> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    // do this here do you dont have to do it later on!
    data = data.replace(",", " ");
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
                _ => l.push(word),
            }
        }
        if l.len() > 0 {
            res.push(l);
        }
    }
    res
}

fn lex_line<'a>(line: &Vec<&str>) -> Res<Vec<Token>, &'a str> {
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

pub fn parse(filename: &str) {
    let file = read_file(filename).unwrap();
    let parsed = get_words(&file);

    let mut tokens: Vec<Token> = Vec::new();
    for line in parsed.iter() {
        match lex_line(line) {
            Ok(mut l) => tokens.append(&mut l),
            Err(e) => panic!("{}",e),
        }
    }

    let mut symbol_table: HashMap<String, usize> = HashMap::new();
    let mut addr = 0;
    for token in tokens.iter() {
        match token {
            Token::Mnemonic(_, _, n_o) => {addr += n_o + 1;},
            Token::Label(s)              => {symbol_table.insert(s.clone(), addr);},
            _                            => ()
        };
    };

    println!("{:?}", symbol_table);
}