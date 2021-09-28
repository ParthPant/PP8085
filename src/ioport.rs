use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct IoPort {
    addr: u8,
    data: u8,
}

impl IoPort {
    pub fn new(addr: u8) -> IoPort {
        IoPort {
            addr: addr,
            data: 0,
        }
    }

    pub fn read(&self) -> u8 {
        self.data
    }

    pub fn write(&mut self, data: u8) {
        self.data = data;
    }

    pub fn get_addr(&self) -> u8 {
        self.addr
    }
}
