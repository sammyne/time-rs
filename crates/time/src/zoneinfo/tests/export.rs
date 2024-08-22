use std::mem;
use std::ops::Deref;

use crate::{Location, LOCAL};

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

fn set_local(l: Location) {
    #[allow(invalid_reference_casting)]
    let local = unsafe { &mut *(LOCAL.deref() as *const Location as *mut Location) };

    let _ = mem::replace(local, l);
}
