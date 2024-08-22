mod errors;
mod location;

use std::env;

pub use errors::*;
pub use location::*;

#[cfg(test)]
mod tests;

lazy_static::lazy_static! {
  static ref ZONEINFO: String = init_zoneinfo();
}

fn init_zoneinfo() -> String {
    env::var("ZONEINFO").unwrap_or_default()
}
