use rand::prelude::*;
use std::{thread::sleep, time::Duration};

pub struct Page([u8; 65_536]);

impl Page {
    pub fn new() -> Self {
        Self([0; 65_536])
    }
}

fn main() {
    loop {
        let n = random::<usize>() % 1024;
        let mut v = Vec::<Page>::with_capacity(n);
        print!("{:?}", (0..v.capacity()));
        for _ in 0..(v.capacity()) {
            v.push(Page::new());
        }
        sleep(Duration::from_millis(1000));
    }
}
