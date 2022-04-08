use once_cell::sync::OnceCell;
use std::collections::HashMap;

static GLOBAL: OnceCell<HashMap<String, Vec<u8>>> = OnceCell::new();

pub fn _initialize() {
    GLOBAL.set(HashMap::new()).unwrap();
}
