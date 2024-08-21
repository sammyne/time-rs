use std::env;

#[test]
fn env_tz_usage() {
    const ENV: &str = "TZ";

    let _defer = EnvKeeper {
        k: ENV.to_string(),
        v: env::var(ENV).ok(),
    };
}

struct EnvKeeper {
    pub k: String,
    pub v: Option<String>,
}

impl Drop for EnvKeeper {
    fn drop(&mut self) {
        match self.v.as_ref() {
            Some(v) => env::set_var(&self.k, v),
            None => env::remove_var(&self.k),
        }
    }
}
