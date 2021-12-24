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

const HEIGHT: f64 = 1024.0;
const WIDTH: f64 = 512.0;

fn main() {
    println!("theta,size");

    // loop {
    for i in 0..10000000 {
        // let start = Instant::now();
        let theta = (i as f64 * PI) / WIDTH;
        let size = HEIGHT * theta.sin() + HEIGHT;
        println!("{},{}", theta, size);
        // println!("{},{}", chrono::Local::now().to_string(), size);
        // sleep(Duration::from_millis(1000) - (Instant::now() - start));
    }
}
