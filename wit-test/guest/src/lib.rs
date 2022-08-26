use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use rand::Rng;
use std::{collections::HashMap, sync::Mutex};
use ulid::Ulid;

wit_bindgen_rust::export!("../wits/proxy.wit");

static TTL: Lazy<Duration> = Lazy::new(|| Duration::minutes(5));

static CACHE: Lazy<Mutex<HashMap<String, DateTime<Utc>>>> = Lazy::new(|| {
    let cache = HashMap::new();
    Mutex::new(cache)
});

struct Proxy;

impl Proxy {
    fn get_new_token() -> String {
        let token = Ulid::new().to_string();
        token + generate_random_string(1024 - ulid::ULID_LEN).as_str()
    }

    fn login() -> String {
        let token = Proxy::get_new_token();

        let mut c = CACHE.lock().unwrap();
        let _ = c.insert(token.clone(), Utc::now() + *TTL);

        token
    }

    fn logout(token: &str) {
        let mut c = CACHE.lock().unwrap();
        c.remove(token);
    }
}

impl proxy::Proxy for Proxy {
    fn onhttp(path: String, auth: String, _method: String) -> String {
        match path.as_str().trim() {
            "/login" => Self::login(),
            "/logout" => {
                Self::logout(&auth);
                // CACHE.lock().unwrap().len().to_string()
                String::from("OK")
            }
            // TODO
            _ => "NOT IMPLEMENTED".into(),
        }
    }

    fn ontick() -> String {
        todo!()
    }
}

fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
