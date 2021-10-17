use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use ttl_cache::TtlCache;

pub fn generate_token() -> String {
    let mut rng = thread_rng();
    let mut output = String::new();
    for i in 0..5 {
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(4)
            .collect();
        output += &chars;

        if i != 4 {
            output += "-";
        }
    }

    output
}

pub fn is_token_valid(token: String, cache: TtlCache<i32, String>) -> bool {
    for (_k, v) in cache.clone().iter() {
        if v.to_owned().eq(&token) {
            return true;
        }
    }
    false
}


pub fn invalidate_token(token: String, cache: &mut TtlCache<i32, String>) -> Result<(), ()> {
    for (k, v) in cache.clone().iter() {
        if v.to_owned().eq(&token) {
            cache.remove(k);
            return Ok(());
        }
    }

    Err(())
}