struct Page([u8; 65_536]);

impl Page {
    fn new() -> Self {
        Self([0; 65_536])
    }
}

fn main() {
    let mut vv = Vec::<Vec<Page>>::with_capacity(0);
    loop {
        let mut v = Vec::<Page>::with_capacity(0);
        for _ in 0..256 {
            v.push(Page::new());
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        vv.push(v);
    }
}
