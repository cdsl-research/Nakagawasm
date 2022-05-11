use std::thread::sleep;

struct Page([u8; 65_536]);

impl Page {
    fn new() -> Self {
        Self([0; 65_536])
    }
}

fn main() {
    loop {
        let page = Box::new(Page::new());

        // DO NOT enable optimization when you compile this file.
        // Because this `Box::leak()` will be erased.
        Box::leak(page);

        sleep(std::time::Duration::from_millis(1));
    }
}
