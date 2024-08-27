use crate::zoneinfo::{Rule, RuleKind};
use crate::PLATFORM_ZONE_SOURCES;

lazy_static::lazy_static! {
  pub static ref ORIG_PLATFORM_ZONE_SOURCES: Vec<&'static str> = crate::PLATFORM_ZONE_SOURCES.clone();
}

pub fn disable_platform_sources() -> impl FnMut() {
    renew_platform_zone_sources([]);

    recover_platform_sources
}

fn renew_platform_zone_sources<T: Into<Vec<&'static str>>>(v: T) {
    #[allow(invalid_reference_casting)]
    let w = unsafe { &mut *(PLATFORM_ZONE_SOURCES.as_ref() as *const Vec<&'static str> as *mut Vec<&'static str>) };
    *w = v.into();
}

fn recover_platform_sources() {
    renew_platform_zone_sources(ORIG_PLATFORM_ZONE_SOURCES.as_slice());
}

impl Rule {
    pub fn julian(day: isize, time: isize) -> Self {
        Self {
            kind: RuleKind::Julian { day },
            time,
        }
    }

    pub fn doy(day: isize, time: isize) -> Self {
        Self {
            kind: RuleKind::DOY { day },
            time,
        }
    }

    pub fn month_week_day(mon: isize, week: isize, day: isize, time: isize) -> Self {
        Self {
            kind: RuleKind::MonthWeekDay { mon, week, day },
            time,
        }
    }
}
