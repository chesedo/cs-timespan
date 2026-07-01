use cs_timespan::{NegativeTimeSpan, TimeSpan};

// ── From<std::time::Duration> for TimeSpan ────────────────────────────────────

#[test]
fn from_duration_zero() {
    assert_eq!(TimeSpan::from(std::time::Duration::ZERO), TimeSpan::ZERO);
}

#[test]
fn from_duration_one_second() {
    assert_eq!(
        TimeSpan::from(std::time::Duration::from_secs(1)),
        TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND),
    );
}

#[test]
fn from_duration_one_tick() {
    // 100 ns = 1 tick
    assert_eq!(
        TimeSpan::from(std::time::Duration::from_nanos(100)),
        TimeSpan::from_ticks(1),
    );
}

#[test]
fn from_duration_sub_tick_rounds_down() {
    // 99 ns < 1 tick; truncates to zero
    assert_eq!(
        TimeSpan::from(std::time::Duration::from_nanos(99)),
        TimeSpan::ZERO,
    );
}

#[test]
fn from_duration_large_saturates_to_max() {
    // Duration::MAX vastly exceeds TimeSpan::MAX_VALUE; must saturate.
    assert_eq!(
        TimeSpan::from(std::time::Duration::MAX),
        TimeSpan::MAX_VALUE
    );
}

// ── TryFrom<TimeSpan> for std::time::Duration ─────────────────────────────────

#[test]
fn try_from_timespan_zero() {
    assert_eq!(
        std::time::Duration::try_from(TimeSpan::ZERO),
        Ok(std::time::Duration::ZERO),
    );
}

#[test]
fn try_from_timespan_positive() {
    assert_eq!(
        std::time::Duration::try_from(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)),
        Ok(std::time::Duration::from_secs(1)),
    );
}

#[test]
fn try_from_timespan_negative_is_err() {
    assert_eq!(
        std::time::Duration::try_from(TimeSpan::from_ticks(-1)),
        Err(NegativeTimeSpan),
    );
    assert_eq!(
        std::time::Duration::try_from(TimeSpan::MIN_VALUE),
        Err(NegativeTimeSpan),
    );
}

#[test]
fn try_from_timespan_max_value() {
    let d = std::time::Duration::try_from(TimeSpan::MAX_VALUE).unwrap();
    // MAX_VALUE ticks × 100 ns = 922_337_203_685 s + 477_580_700 ns remainder
    assert_eq!(d.as_secs(), 922_337_203_685);
    assert_eq!(d.subsec_nanos(), 477_580_700);
}

// ── chrono feature: From<TimeDelta> and From<TimeSpan> for TimeDelta ──────────

#[cfg(feature = "chrono")]
mod chrono_conversions {
    use chrono::TimeDelta;
    use cs_timespan::TimeSpan;

    #[test]
    fn from_timedelta_zero() {
        assert_eq!(TimeSpan::from(TimeDelta::zero()), TimeSpan::ZERO);
    }

    #[test]
    fn from_timedelta_positive() {
        assert_eq!(
            TimeSpan::from(TimeDelta::seconds(1)),
            TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND),
        );
    }

    #[test]
    fn from_timedelta_negative() {
        assert_eq!(
            TimeSpan::from(TimeDelta::seconds(-1)),
            TimeSpan::from_ticks(-TimeSpan::TICKS_PER_SECOND),
        );
    }

    #[test]
    fn from_timedelta_fractional_seconds() {
        // -1.5 s = -15_000_000 ticks
        let delta = TimeDelta::seconds(-1) + TimeDelta::nanoseconds(-500_000_000);
        assert_eq!(TimeSpan::from(delta), TimeSpan::from_ticks(-15_000_000));
    }

    #[test]
    fn from_timedelta_sub_tick_precision_is_truncated() {
        // 99 ns is less than 1 tick (100 ns); truncates to zero
        assert_eq!(TimeSpan::from(TimeDelta::nanoseconds(99)), TimeSpan::ZERO);
    }

    #[test]
    fn timespan_to_timedelta_zero() {
        assert_eq!(TimeDelta::from(TimeSpan::ZERO), TimeDelta::zero());
    }

    #[test]
    fn timespan_to_timedelta_positive() {
        assert_eq!(
            TimeDelta::from(TimeSpan::from_ticks(TimeSpan::TICKS_PER_SECOND)),
            TimeDelta::seconds(1),
        );
    }

    #[test]
    fn timespan_to_timedelta_negative() {
        assert_eq!(
            TimeDelta::from(TimeSpan::from_ticks(-TimeSpan::TICKS_PER_SECOND)),
            TimeDelta::seconds(-1),
        );
    }

    #[test]
    fn timespan_to_timedelta_roundtrip() {
        // TimeSpan → TimeDelta → TimeSpan must be lossless (TimeSpan has 100 ns precision).
        for ticks in [0i64, 1, -1, 12_345_678, -9_876_543, i64::MAX / 2] {
            let ts = TimeSpan::from_ticks(ticks);
            let roundtrip = TimeSpan::from(TimeDelta::from(ts));
            assert_eq!(roundtrip, ts, "roundtrip failed for ticks={ticks}");
        }
    }
}
