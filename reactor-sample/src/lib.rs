use once_cell::sync::OnceCell;

static GLOBAL: OnceCell<i32> = OnceCell::new();

#[no_mangle]
pub extern "C" fn f() {
    println!("{}", GLOBAL.get_or_init(|| 0));
}

#[no_mangle]
pub extern "C" fn _initialize() {
    GLOBAL.set(16).unwrap();
}
