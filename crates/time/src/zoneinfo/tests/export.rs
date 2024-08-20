use std::ops::Deref;

use std::mem;

use crate::{Location, LOCAL};

pub fn reset_local() {
  #[allow(invalid_reference_casting)]
  let local = unsafe { &mut *(LOCAL.deref() as *const Location as *mut Location) };

  let _ = mem::replace(local, Location::default());
}