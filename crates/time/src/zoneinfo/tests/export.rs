use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::{env, mem};

use crate::zoneinfo::ZONEINFO;
use crate::{Location, LOCAL};

pub struct Defer<F: FnMut()>(F);

#[derive(Default)]
pub struct Testing {
    env_vars: HashMap<OsString, Option<String>>,
    clean_ups: Vec<Box<dyn FnOnce()>>,
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

impl Testing {
    pub fn clean_up<F>(&mut self, f: F)
    where
        F: FnOnce() + 'static,
    {
        self.clean_ups.push(Box::new(f))
    }

    pub fn setenv<K, V>(&mut self, k: K, v: V)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env_vars.insert(k.as_ref().to_owned(), env::var(k.as_ref()).ok());

        env::set_var(k, v);
    }
}

impl Drop for Testing {
    fn drop(&mut self) {
        for (k, v) in self.env_vars.iter() {
            match v {
                Some(v) => env::set_var(k, v),
                None => env::remove_var(k),
            }
        }

        for v in self.clean_ups.drain(..) {
            v();
        }
    }
}

pub fn force_us_pacific_for_testing() {
    reset_local();
    init_testing_zone();
}

pub fn init_testing_zone() {
    let mut z = crate::zoneinfo::load_location("America/Los_Angeles", &[]).expect("cannot load America/Los_Angeles");
    z.name = "Local".to_string();
    set_local(z);
}

pub fn reset_local() {
    set_local(Location::local());
}

pub fn reset_zoneinfo() {
    #[allow(invalid_reference_casting)]
    let zoneinfo = unsafe { &mut *(ZONEINFO.deref() as *const String as *mut String) };
    *zoneinfo = crate::zoneinfo::init_zoneinfo();
}

pub fn zoneinfo() -> String {
    ZONEINFO.clone()
}

fn set_local(l: Location) {
    #[allow(invalid_reference_casting)]
    let local = unsafe { &mut *(LOCAL.deref() as *const Location as *mut Location) };

    let _ = mem::replace(local, l);
}
