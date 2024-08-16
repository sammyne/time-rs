use std::time::SystemTime;

/// 返回自 1970-01-01 00:00:00 UTC 至今的秒数和纳秒数。
pub fn now() -> (i64, i32) {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(v) => (v.as_secs() as i64, v.subsec_nanos() as i32),
        Err(v) => {
            let d = v.duration();
            let s = d.as_secs() as i64 * -1;
            let ns = d.subsec_nanos() as i32 * -1;
            (s, ns)
        }
    }
}
