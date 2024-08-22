use std::env;
use std::path::Path;

use super::export;

#[test]
fn env_tz_usage() {
    const ENV: &str = "TZ";

    let _defer = EnvKeeper {
        k: ENV.to_string(),
        v: env::var(ENV).ok(),
    };
    let _d: Defer<_> = export::force_us_pacific_for_testing.into();

    let local_zone_name: &str = if Path::new("/etc/localtime").exists() {
        "Local"
    } else {
        "UTC"
    };

    struct Case {
        nil_flag: bool,
        tz: &'static str,
        local: &'static str,
    }
    impl Case {
        fn new(nil_flag: bool, tz: &'static str, local: &'static str) -> Self {
            Self { nil_flag, tz, local }
        }
    }

    let cases = vec![
        // no $TZ means use the system default /etc/localtime.
        Case::new(true, "", local_zone_name),
        // $TZ="" means use UTC.
        Case::new(false, "", "UTC"),
        Case::new(false, ":", "UTC"),
        Case::new(false, "Asia/Shanghai", "Asia/Shanghai"),
        Case::new(false, ":Asia/Shanghai", "Asia/Shanghai"),
        Case::new(false, "/etc/localtime", local_zone_name),
        Case::new(false, ":/etc/localtime", local_zone_name),
    ];

    for (i, c) in cases.into_iter().enumerate() {
        if c.nil_flag {
            env::remove_var(ENV);
        } else {
            env::set_var(ENV, c.tz);
        }
        // lazy_static 和 golang 的 sync.Once 的机制不一样，因此需要放到设置环境变量之后。
        export::reset_local();
        assert_eq!(
            crate::LOCAL.to_string(),
            c.local,
            "#{} invalid Local location name for '{}'",
            i,
            c.tz
        );
    }

    // The file may not exist on Solaris 2 and IRIX 6.
    const PATH: &str = "/usr/share/zoneinfo/Asia/Shanghai";
    env::set_var(ENV, PATH);
    if !Path::new(PATH).exists() {
        assert_eq!(crate::LOCAL.to_string(), "UTC", "invalid path should fallback to UTC");
        return;
    }
    export::reset_local();
    assert_eq!(crate::LOCAL.to_string(), PATH, "custom path should lead to path itself");

    // TODO: add case against Shanghai

    env::set_var(ENV, format!(":{PATH}"));
    export::reset_local();
    assert_eq!(crate::LOCAL.to_string(), PATH, "custom path should lead to path itself");

    env::set_var(ENV, PATH.split_at(PATH.len() - 1).0);
    export::reset_local();
    assert_eq!(crate::LOCAL.to_string(), "UTC", "invalid path should fallback to UTC");
}

pub struct Defer<F: FnMut()>(F);

struct EnvKeeper {
    pub k: String,
    pub v: Option<String>,
}

impl<F: FnMut()> Drop for Defer<F> {
    fn drop(&mut self) {
        self.0()
    }
}

impl<F: FnMut()> From<F> for Defer<F> {
    fn from(value: F) -> Self {
        Self(value)
    }
}

impl Drop for EnvKeeper {
    fn drop(&mut self) {
        match self.v.as_ref() {
            Some(v) => env::set_var(&self.k, v),
            None => env::remove_var(&self.k),
        }
    }
}
