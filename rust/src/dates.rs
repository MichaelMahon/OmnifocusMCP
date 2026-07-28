//! Date inputs from MCP clients are frequently bare calendar dates (`2026-08-01`).
//!
//! JXA evaluates `new Date("2026-08-01")` using the ECMA-262 date-only form, which is
//! interpreted as **UTC** midnight. In any non-UTC zone that lands on the wrong wall-clock
//! day: in `America/New_York` (UTC-4) it becomes 20:00 on July 31, so a task deferred to
//! "Aug 1" actually becomes available a day early.
//!
//! The date-*time* form (`2026-08-01T00:00:00`, no trailing `Z`) is interpreted as local
//! time instead, which is what a user means by "defer to Aug 1". So we widen bare dates to
//! local midnight before handing them to JXA, and leave every other form untouched.

/// True when `value` is exactly `YYYY-MM-DD` with no time component.
fn is_bare_calendar_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes.iter().enumerate().all(|(i, b)| match i {
        4 | 7 => *b == b'-',
        _ => b.is_ascii_digit(),
    })
}

/// Widen a bare `YYYY-MM-DD` to `YYYY-MM-DDT00:00:00` so JXA reads it as local midnight.
///
/// Anything already carrying a time, an offset, or a `Z` is passed through verbatim — those
/// forms are unambiguous and must keep their explicit meaning.
pub fn normalize_date_input(value: &str) -> String {
    let trimmed = value.trim();
    if is_bare_calendar_date(trimmed) {
        format!("{trimmed}T00:00:00")
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_calendar_date_is_widened_to_local_midnight() {
        assert_eq!(normalize_date_input("2026-08-01"), "2026-08-01T00:00:00");
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_before_widening() {
        assert_eq!(normalize_date_input("  2026-08-01  "), "2026-08-01T00:00:00");
    }

    #[test]
    fn explicit_local_datetime_is_untouched() {
        assert_eq!(
            normalize_date_input("2026-08-01T09:30:00"),
            "2026-08-01T09:30:00"
        );
    }

    #[test]
    fn explicit_utc_instant_is_untouched() {
        // A trailing Z is an intentional UTC instant; widening it would change its meaning.
        assert_eq!(
            normalize_date_input("2026-08-01T00:00:00Z"),
            "2026-08-01T00:00:00Z"
        );
    }

    #[test]
    fn explicit_offset_is_untouched() {
        assert_eq!(
            normalize_date_input("2026-08-01T00:00:00-04:00"),
            "2026-08-01T00:00:00-04:00"
        );
    }

    #[test]
    fn non_date_strings_pass_through() {
        for value in ["", "tomorrow", "2026-8-1", "20260801", "2026-08-01x"] {
            assert_eq!(normalize_date_input(value), value.trim());
        }
    }
}
