use std::{
    thread::sleep,
    time::{Duration, Instant}, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}
};
use rand::random;

fn f() {
    let fname = random::<u32>() % 1024;
    let data = std::fs::read(fname.to_string()).unwrap();
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    let hash = s.finish();
    println!("{},{},{},{}", chrono::Local::now().to_rfc3339(), fname, data.len(), hash);
}

fn main() {
    loop {
        let start = Instant::now();
        f();
        sleep(Duration::from_secs(10) - (Instant::now() - start));
    }
}
