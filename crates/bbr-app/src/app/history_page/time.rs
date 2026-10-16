//! Blueprint D10: the History page's clock — local days, the time of a
//! visit, and the day-group titles.
//!
//! Visits carry `last_visited` in milliseconds since the Unix epoch. Stores
//! written before D10 carry a small monotonic counter there instead; such a
//! visit has no known time ([`Clock::dated`] is false), sorts below every
//! dated one and is listed under "Earlier".

/// Anything below this (September 2001 in milliseconds) is a pre-D10
/// recency counter, not a time.
pub(crate) const DATED_FLOOR_MS: u64 = 1_000_000_000_000;

const DAY_S: i64 = 86_400;
const WEEKDAYS: [&str; 7] = [
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Now plus the local offset from UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clock {
    pub now_ms: u64,
    pub offset_s: i64,
}

/// Testkit only: the pinned display clock (`SetDisplayClock`), so dated
/// History evidence never depends on the machine's time or zone.
#[cfg(feature = "testkit")]
static PINNED: std::sync::Mutex<Option<Clock>> = std::sync::Mutex::new(None);

/// The largest local UTC offset a zone uses (+14 h, Line Islands).
#[cfg(feature = "testkit")]
pub(crate) const MAX_OFFSET_S: i64 = 14 * 3600;

impl Clock {
    /// Testkit only: pin (or with `None` release) the display clock.
    #[cfg(feature = "testkit")]
    pub(crate) fn pin(clock: Option<Clock>) {
        *PINNED.lock().unwrap_or_else(|poison| poison.into_inner()) = clock;
    }

    /// The system clock in the current time zone (the pinned display clock
    /// instead while a testkit run holds one).
    pub(crate) fn system() -> Clock {
        #[cfg(feature = "testkit")]
        if let Some(clock) = *PINNED.lock().unwrap_or_else(|poison| poison.into_inner()) {
            return clock;
        }
        #[cfg(target_os = "macos")]
        let offset_s = bbr_web_macos::local_utc_offset_seconds();
        #[cfg(not(target_os = "macos"))]
        let offset_s = 0;
        Clock {
            now_ms: crate::store::history_ops::now_ms(),
            offset_s,
        }
    }

    /// Whether `ms` is a time (not a pre-D10 counter).
    pub(crate) fn dated(ms: u64) -> bool {
        ms >= DATED_FLOOR_MS
    }

    fn local_s(&self, ms: u64) -> i64 {
        (ms / 1000) as i64 + self.offset_s
    }

    /// The local day number of `ms` (days since 1970-01-01, local).
    pub(crate) fn day(&self, ms: u64) -> i64 {
        self.local_s(ms).div_euclid(DAY_S)
    }

    /// Today's local day number.
    pub(crate) fn today(&self) -> i64 {
        self.day(self.now_ms)
    }

    /// `HH:MM`, 24-hour, local.
    pub(crate) fn time_label(&self, ms: u64) -> String {
        let seconds = self.local_s(ms).rem_euclid(DAY_S);
        format!("{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60)
    }

    /// "Today", "Yesterday", or "Monday, 21 September" (with the year when
    /// it is not this year).
    pub(crate) fn day_title(&self, day: i64) -> String {
        let today = self.today();
        if day == today {
            return "Today".into();
        }
        if day == today - 1 {
            return "Yesterday".into();
        }
        let (year, month, dom) = civil_from_days(day);
        let (this_year, _, _) = civil_from_days(today);
        let weekday = WEEKDAYS[day.rem_euclid(7) as usize];
        let month = MONTHS[(month - 1) as usize];
        if year == this_year {
            format!("{weekday}, {dom} {month}")
        } else {
            format!("{weekday}, {dom} {month} {year}")
        }
    }
}

impl Clock {
    /// "3 Mar 2027" for a Unix time in seconds (blueprint D12's
    /// certificate expiry).
    pub(crate) fn short_date(&self, unix_seconds: i64) -> String {
        let day = (unix_seconds + self.offset_s).div_euclid(DAY_S);
        let (year, month, dom) = civil_from_days(day);
        format!("{dom} {} {year}", &MONTHS[(month - 1) as usize][..3])
    }
}

/// Days since 1970-01-01 to `(year, month 1–12, day 1–31)` (Howard
/// Hinnant's civil-from-days).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-09-29 12:00:00 UTC.
    const NOON: u64 = 1_790_683_200_000;

    #[cfg(feature = "testkit")]
    #[test]
    fn a_pinned_display_clock_replaces_the_system_until_released() {
        // No other unit test reads the system clock; this one restores it.
        let pinned = Clock {
            now_ms: NOON,
            offset_s: 2 * 3600,
        };
        Clock::pin(Some(pinned));
        assert_eq!(Clock::system(), pinned);
        assert_eq!(Clock::system().day_title(Clock::system().today()), "Today");
        Clock::pin(None);
        assert_ne!(Clock::system().now_ms, NOON, "the system clock again");
    }

    #[test]
    fn civil_dates_round_trip_known_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(20_725), (2026, 9, 29));
    }

    #[test]
    fn days_times_and_titles_follow_the_local_offset() {
        let utc = Clock {
            now_ms: NOON,
            offset_s: 0,
        };
        assert_eq!(utc.time_label(NOON), "12:00");
        assert_eq!(utc.day_title(utc.today()), "Today");
        assert_eq!(utc.day_title(utc.today() - 1), "Yesterday");
        assert_eq!(utc.day_title(utc.today() - 2), "Sunday, 27 September");
        assert_eq!(utc.day_title(19_723), "Monday, 1 January 2024");
        // Thirteen hours east it is already 01:00 the next day.
        let east = Clock {
            now_ms: NOON,
            offset_s: 13 * 3600,
        };
        assert_eq!(east.time_label(NOON), "01:00");
        assert_eq!(east.day(NOON), utc.day(NOON) + 1);
    }

    #[test]
    fn short_dates_name_day_month_and_year() {
        let utc = Clock {
            now_ms: NOON,
            offset_s: 0,
        };
        assert_eq!(utc.short_date(2_106_063_227), "26 Sep 2036");
        assert_eq!(utc.short_date(0), "1 Jan 1970");
    }

    #[test]
    fn pre_d10_counters_are_not_times() {
        assert!(!Clock::dated(42));
        assert!(Clock::dated(NOON));
    }
}
