pub struct Memory {
    data: Vec<u8>,
    size: u16,
}

impl Memory {
    pub fn new(size: u16) -> Memory {
        Memory {
            data: vec![0; size as usize],
            size: size,
        }
    }
    
    pub fn new_from(bin: &Vec<u8>, size: usize) -> Memory {
        assert!(bin.len() < size);            
        let mut res = Memory {
            data: vec![0; size as usize],
            size: size as u16,
        };
        for i in 0..bin.len() {
             res.data[i] = bin[i];
        };
        res
    }
    
    pub fn size(&self) -> u16 {
        self.size
    }

    /// write to a given address in the memory
    pub fn write(&mut self, addr: u16, content: u8) {
        if addr < self.size {
            self.data[addr as usize] = content;
        } else {
            panic!("Memory overflow {:#02x}/{:#02x}", addr, self.size);
        }
    }

    // return as optional result here
    /// read from a given address in the memory
    pub fn read(&self, addr: u16) -> u8 {
        if addr < self.size {
            return self.data[addr as usize];
        } else {
            panic!("Memory overflow");
        }
    }

    pub fn display(&self, addr: u16) {
        if addr < self.size {
            println!("{:#02x} : {:#02x}", addr, self.data[addr as usize]);
        } else {
            println!("Memory out of range");
        }
    }
}
