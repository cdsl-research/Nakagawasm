use rand::prelude::*;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    loop {
        let n: usize = random::<usize>() % 1024;
        println!("{}", n);
        let mut b = Box::new(Vec::<[u8; 32 * 1024]>::with_capacity(n));
        b.iter_mut()
            .for_each(|e| e.iter_mut().for_each(|x| *x += 1));

        sleep(Duration::from_secs(5));
    }
}
