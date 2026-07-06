// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GetPerformanceChartRequest;
use anki_proto::brainlift::GetPerformanceChartResponse;
use anki_proto::brainlift::PerformanceChartBucket;
use anki_proto::brainlift::PerformanceChartHorizon;
use chrono::Datelike;
use chrono::Duration;
use chrono::FixedOffset;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Weekday as ChronoWeekday;

use crate::collection::Collection;
use crate::config::Weekday;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::storage::PerformanceAttemptChartRow;
use crate::prelude::TimestampSecs;

const SECS_PER_HOUR: i64 = 3_600;
const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_WEEK: i64 = SECS_PER_DAY * 7;

#[derive(Debug, Clone)]
struct BucketSpec {
    label: String,
    start_secs: i64,
    end_secs: i64,
}

impl Collection {
    pub fn gre_atlas_get_performance_chart(
        &mut self,
        req: GetPerformanceChartRequest,
    ) -> Result<GetPerformanceChartResponse> {
        let topic_prefix = req.topic_prefix.as_str();
        let timing = self.timing_today()?;
        let offset = self.local_utc_offset_for_user()?;
        let first_day_of_week = self.get_first_day_of_week();
        let storage = gre_atlas_storage(self)?;
        let has_any_attempts = storage.has_performance_attempts(topic_prefix)?;

        let next_day_at = timing.next_day_at.0;
        let today_start = next_day_at - SECS_PER_DAY;

        let horizon = req.horizon();
        let specs = match horizon {
            PerformanceChartHorizon::PerformanceChartHorizon1d => {
                hourly_bucket_specs(today_start, next_day_at)
            }
            PerformanceChartHorizon::PerformanceChartHorizon3d => {
                daily_bucket_specs(next_day_at, 3, offset)
            }
            PerformanceChartHorizon::PerformanceChartHorizon7d => {
                daily_bucket_specs(next_day_at, 7, offset)
            }
            PerformanceChartHorizon::PerformanceChartHorizon30d => weekly_bucket_specs(
                next_day_at - 30 * SECS_PER_DAY,
                next_day_at,
                offset,
                first_day_of_week,
            ),
            PerformanceChartHorizon::All => {
                let earliest = storage.earliest_attempt_secs(topic_prefix)?;
                match earliest {
                    Some(stamp) => monthly_bucket_specs(stamp.0, next_day_at, offset),
                    None => Vec::new(),
                }
            }
            PerformanceChartHorizon::Unspecified => weekly_bucket_specs(
                next_day_at - 30 * SECS_PER_DAY,
                next_day_at,
                offset,
                first_day_of_week,
            ),
        };

        if specs.is_empty() {
            return Ok(GetPerformanceChartResponse {
                buckets: Vec::new(),
                has_any_attempts,
            });
        }

        let since = specs[0].start_secs;
        let attempts =
            storage.attempts_in_range(TimestampSecs(since), TimestampSecs(next_day_at), topic_prefix)?;
        let buckets = aggregate_attempts(&specs, &attempts);

        Ok(GetPerformanceChartResponse {
            buckets,
            has_any_attempts,
        })
    }
}

fn hourly_bucket_specs(today_start: i64, next_day_at: i64) -> Vec<BucketSpec> {
    debug_assert_eq!(next_day_at - today_start, SECS_PER_DAY);
    (0..24)
        .map(|hour| {
            let start_secs = today_start + hour * SECS_PER_HOUR;
            BucketSpec {
                label: hour_label(hour as u32),
                start_secs,
                end_secs: start_secs + SECS_PER_HOUR,
            }
        })
        .collect()
}

fn daily_bucket_specs(next_day_at: i64, day_count: u32, offset: FixedOffset) -> Vec<BucketSpec> {
    let day_count = day_count as i64;
    (0..day_count)
        .map(|index| {
            let start_secs = next_day_at - (day_count - index) * SECS_PER_DAY;
            BucketSpec {
                label: day_label(TimestampSecs(start_secs), offset),
                start_secs,
                end_secs: start_secs + SECS_PER_DAY,
            }
        })
        .collect()
}

fn weekly_bucket_specs(
    earliest_secs: i64,
    next_day_at: i64,
    offset: FixedOffset,
    first_day_of_week: Weekday,
) -> Vec<BucketSpec> {
    let week_start = week_start_for_stamp(earliest_secs, offset, first_day_of_week);
    let current_week_start = week_start_for_stamp(next_day_at - 1, offset, first_day_of_week);
    let mut specs = Vec::new();
    let mut start_secs = week_start;
    while start_secs <= current_week_start {
        let end_secs = start_secs + SECS_PER_WEEK;
        specs.push(BucketSpec {
            label: week_label(TimestampSecs(start_secs), TimestampSecs(end_secs - 1), offset),
            start_secs,
            end_secs,
        });
        start_secs += SECS_PER_WEEK;
    }
    specs
}

/// Calendar-month buckets from the earliest attempt's month through the
/// current month. Used by the All-time horizon.
fn monthly_bucket_specs(
    earliest_secs: i64,
    next_day_at: i64,
    offset: FixedOffset,
) -> Vec<BucketSpec> {
    let (Ok(first), Ok(current)) = (
        TimestampSecs(earliest_secs).datetime(offset),
        TimestampSecs(next_day_at - 1).datetime(offset),
    ) else {
        return Vec::new();
    };
    let (mut year, mut month) = (first.year(), first.month());
    let (end_year, end_month) = (current.year(), current.month());
    let mut specs = Vec::new();
    loop {
        let start_secs = month_start_ts(offset, year, month);
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let end_secs = month_start_ts(offset, next_year, next_month);
        specs.push(BucketSpec {
            label: month_label(year, month),
            start_secs,
            end_secs,
        });
        if (year, month) == (end_year, end_month) || specs.len() >= 600 {
            break;
        }
        year = next_year;
        month = next_month;
    }
    specs
}

fn month_start_ts(offset: FixedOffset, year: i32, month: u32) -> i64 {
    offset
        .with_ymd_and_hms(year, month, 1, 0, 0, 0)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

fn month_label(year: i32, month: u32) -> String {
    const NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let name = NAMES.get((month.saturating_sub(1)) as usize).unwrap_or(&"");
    format!("{name} {year}")
}

fn aggregate_attempts(
    specs: &[BucketSpec],
    attempts: &[PerformanceAttemptChartRow],
) -> Vec<PerformanceChartBucket> {
    specs
        .iter()
        .map(|spec| {
            let mut correct = 0u32;
            let mut incorrect = 0u32;
            for attempt in attempts {
                let stamp = attempt.answered_at_secs.0;
                if stamp >= spec.start_secs && stamp < spec.end_secs {
                    if attempt.correct {
                        correct += 1;
                    } else {
                        incorrect += 1;
                    }
                }
            }
            let questions = correct + incorrect;
            PerformanceChartBucket {
                label: spec.label.clone(),
                start_secs: spec.start_secs,
                end_secs: spec.end_secs,
                questions,
                correct,
                incorrect,
                accuracy: if questions > 0 {
                    Some(correct as f32 / questions as f32)
                } else {
                    None
                },
            }
        })
        .collect()
}

fn hour_label(hour: u32) -> String {
    hour.to_string()
}

fn day_label(stamp: TimestampSecs, offset: FixedOffset) -> String {
    stamp
        .datetime(offset)
        .map(|dt| format!("{} {}", dt.format("%b"), dt.day()))
        .unwrap_or_else(|_| stamp.date_string())
}

fn week_label(week_start: TimestampSecs, week_end_inclusive: TimestampSecs, offset: FixedOffset) -> String {
    let Ok(start) = week_start.datetime(offset) else {
        return format!("Week of {}", week_start.date_string());
    };
    let Ok(end) = week_end_inclusive.datetime(offset) else {
        return format!("Week of {} {}", start.format("%b"), start.day());
    };
    if start.month() == end.month() {
        format!(
            "Week of {} {}–{}",
            start.format("%b"),
            start.day(),
            end.day()
        )
    } else {
        format!(
            "{} {}–{} {}",
            start.format("%b"),
            start.day(),
            end.format("%b"),
            end.day()
        )
    }
}

fn week_start_for_stamp(stamp: i64, offset: FixedOffset, first_day_of_week: Weekday) -> i64 {
    let Ok(dt) = TimestampSecs(stamp).datetime(offset) else {
        return stamp - stamp.rem_euclid(SECS_PER_WEEK);
    };
    let week_start_day = config_weekday_to_chrono(first_day_of_week);
    let days_since_start = (dt.weekday().num_days_from_monday() as i64
        - week_start_day.num_days_from_monday() as i64)
        .rem_euclid(7);
    let start_dt = dt - Duration::days(days_since_start);
    start_dt
        .with_hour(0)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .map(|dt| dt.timestamp())
        .unwrap_or(stamp)
}

fn config_weekday_to_chrono(day: Weekday) -> ChronoWeekday {
    match day {
        Weekday::Sunday => ChronoWeekday::Sun,
        Weekday::Monday => ChronoWeekday::Mon,
        Weekday::Friday => ChronoWeekday::Fri,
        Weekday::Saturday => ChronoWeekday::Sat,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn offset_west_hours(hours: i32) -> FixedOffset {
        FixedOffset::west_opt(hours * 3_600).unwrap()
    }

    fn attempt(stamp: i64, correct: bool) -> PerformanceAttemptChartRow {
        PerformanceAttemptChartRow {
            answered_at_secs: TimestampSecs(stamp),
            correct,
        }
    }

    #[test]
    fn hourly_specs_cover_twenty_four_buckets() {
        let next_day_at = 1_700_010_000i64;
        let today_start = next_day_at - SECS_PER_DAY;
        let specs = hourly_bucket_specs(today_start, next_day_at);

        assert_eq!(specs.len(), 24);
        assert_eq!(specs[0].label, "0");
        assert_eq!(specs[12].label, "12");
        assert_eq!(specs[23].label, "23");
        assert_eq!(specs[0].start_secs, today_start);
        assert_eq!(specs[23].end_secs, next_day_at);
    }

    #[test]
    fn daily_specs_cover_requested_day_count() {
        let offset = offset_west_hours(5);
        let next_day_at = 1_700_086_400i64;
        let specs = daily_bucket_specs(next_day_at, 3, offset);

        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0].start_secs, next_day_at - 3 * SECS_PER_DAY);
        assert_eq!(specs[2].end_secs, next_day_at);
    }

    #[test]
    fn accuracy_is_ratio_of_counts_not_average_of_percentages() {
        let specs = vec![
            BucketSpec {
                label: "Morning".into(),
                start_secs: 100,
                end_secs: 200,
            },
            BucketSpec {
                label: "Afternoon".into(),
                start_secs: 200,
                end_secs: 300,
            },
        ];
        let attempts = vec![
            attempt(120, true),
            attempt(130, true),
            attempt(140, false),
            attempt(220, false),
            attempt(230, false),
        ];
        let buckets = aggregate_attempts(&specs, &attempts);

        assert_eq!(buckets[0].questions, 3);
        assert_eq!(buckets[0].correct, 2);
        assert_eq!(buckets[0].incorrect, 1);
        assert!((buckets[0].accuracy.unwrap() - (2.0 / 3.0)).abs() < 0.0001);

        assert_eq!(buckets[1].questions, 2);
        assert_eq!(buckets[1].correct, 0);
        assert_eq!(buckets[1].incorrect, 2);
        assert!((buckets[1].accuracy.unwrap() - 0.0).abs() < 0.0001);
    }

    #[test]
    fn empty_buckets_keep_zero_questions_and_no_accuracy() {
        let specs = vec![BucketSpec {
            label: "Empty".into(),
            start_secs: 0,
            end_secs: 100,
        }];
        let buckets = aggregate_attempts(&specs, &[]);

        assert_eq!(buckets[0].questions, 0);
        assert_eq!(buckets[0].correct, 0);
        assert_eq!(buckets[0].incorrect, 0);
        assert!(buckets[0].accuracy.is_none());
    }

    #[test]
    fn monthly_specs_span_first_through_current_month() {
        let offset = offset_west_hours(0);
        // 2023-11-15 12:00 UTC → first bucket Nov 2023.
        let earliest = 1_700_049_600i64;
        // ~2.5 months later lands in Jan 2024.
        let next_day_at = earliest + SECS_PER_DAY * 75;
        let specs = monthly_bucket_specs(earliest, next_day_at, offset);

        assert_eq!(specs.first().map(|s| s.label.as_str()), Some("Nov 2023"));
        assert_eq!(specs.last().map(|s| s.label.as_str()), Some("Jan 2024"));
        assert_eq!(specs.len(), 3);
        // Buckets are contiguous month boundaries.
        for pair in specs.windows(2) {
            assert_eq!(pair[0].end_secs, pair[1].start_secs);
        }
    }

    #[test]
    fn weekly_specs_start_on_configured_weekday() {
        let offset = offset_west_hours(0);
        // 2023-11-15 12:00 UTC is a Wednesday.
        let earliest = 1_700_049_600i64;
        let next_day_at = earliest + SECS_PER_WEEK * 3;
        let specs = weekly_bucket_specs(earliest, next_day_at, offset, Weekday::Monday);

        assert!(!specs.is_empty());
        let first_start = TimestampSecs(specs[0].start_secs).datetime(offset).unwrap();
        assert_eq!(first_start.weekday(), ChronoWeekday::Mon);
        assert!(specs[0].label.starts_with("Week of"));
    }
}
