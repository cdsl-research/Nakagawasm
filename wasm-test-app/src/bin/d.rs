use std::{
    f64::consts::PI,
    thread::sleep,
    time::{Duration, Instant},
};
pub struct Page([u8; 65_536]);

impl Page {
    pub fn new() -> Self {
        Self([0; 65_536])
    }
}

const HEIGHT: f64 = 256.0;
const WIDTH: f64 = 128.0;

fn main() {
    let mut i: usize = 0;
    loop {
        let start = Instant::now();

        let theta = ((i as f64 - WIDTH / 2.0) / WIDTH) * PI;
        let size = HEIGHT * theta.sin() + HEIGHT;
        let size = size as usize;

        let mut v = Vec::<Page>::with_capacity(size);
        for _ in 0..v.capacity() {
            v.push(Page::new());
        }

        i = (i + 1) % HEIGHT as usize;

        println!("{},{}", chrono::Local::now().to_string(), size);
        sleep(Duration::from_millis(1000) - (Instant::now() - start));
    }
}
