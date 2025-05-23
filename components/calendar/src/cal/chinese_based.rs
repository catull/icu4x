// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and traits for use in the Chinese traditional lunar calendar,
//! as well as in related and derived calendars such as the Korean and Vietnamese lunar calendars.

use crate::{
    calendar_arithmetic::{ArithmeticDate, CalendarArithmetic, PrecomputedDataSource},
    error::DateError,
    provider::chinese_based::{ChineseBasedCache, PackedChineseBasedYearInfo},
    types::{MonthCode, MonthInfo},
    Calendar, Iso,
};

use calendrical_calculations::chinese_based::{self, ChineseBased, YearBounds};
use calendrical_calculations::rata_die::RataDie;
use core::marker::PhantomData;
use core::num::NonZeroU8;
use tinystr::tinystr;

/// The trait ChineseBased is used by Chinese-based calendars to perform computations shared by such calendar.
///
/// For an example of how to use this trait, see `impl ChineseBasedWithDataLoading for Chinese` in [`Chinese`].
pub(crate) trait ChineseBasedWithDataLoading: Calendar {
    type CB: ChineseBased;
    /// Get the compiled const data for a ChineseBased calendar; can return `None` if the given year
    /// does not correspond to any compiled data.
    fn get_precomputed_data(&self) -> ChineseBasedPrecomputedData<'_, Self::CB>;
}

/// Chinese-based calendars define DateInner as a calendar-specific struct wrapping ChineseBasedDateInner.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct ChineseBasedDateInner<C: CalendarArithmetic>(pub(crate) ArithmeticDate<C>);

// we want these impls without the `C: Copy/Clone` bounds
impl<C: CalendarArithmetic> Copy for ChineseBasedDateInner<C> {}
impl<C: CalendarArithmetic> Clone for ChineseBasedDateInner<C> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Contains any loaded precomputed data. If constructed with Default, will
/// *not* contain any extra data and will always compute stuff from scratch
#[derive(Default)]
pub(crate) struct ChineseBasedPrecomputedData<'a, CB: ChineseBased> {
    data: Option<&'a ChineseBasedCache<'a>>,
    _cb: PhantomData<CB>,
}

/// Compute ChineseBasedYearInfo for a given extended year
fn compute_cache<CB: ChineseBased>(related_iso: i32) -> ChineseBasedYearInfo {
    let mid_year = calendrical_calculations::iso::fixed_from_iso(related_iso, 7, 1);
    let year_bounds = YearBounds::compute::<CB>(mid_year);
    compute_cache_with_yb::<CB>(related_iso, year_bounds)
}

/// Compute ChineseBasedYearInfo for a given extended year, for which you have already computed the YearBounds
fn compute_cache_with_yb<CB: ChineseBased>(
    related_iso: i32,
    year_bounds: YearBounds,
) -> ChineseBasedYearInfo {
    let packed_data = compute_packed_with_yb::<CB>(related_iso, year_bounds);

    ChineseBasedYearInfo {
        packed_data,
        related_iso,
    }
}

fn compute_packed_with_yb<CB: ChineseBased>(
    related_iso: i32,
    year_bounds: YearBounds,
) -> PackedChineseBasedYearInfo {
    let YearBounds {
        new_year,
        next_new_year,
        ..
    } = year_bounds;
    let (month_lengths, leap_month) =
        chinese_based::month_structure_for_year::<CB>(new_year, next_new_year);

    let iso_ny = calendrical_calculations::iso::fixed_from_iso(related_iso, 1, 1);

    // +1 because `new_year - iso_ny` is zero-indexed, but `FIRST_NY` is 1-indexed
    let ny_offset = new_year - iso_ny - i64::from(PackedChineseBasedYearInfo::FIRST_NY) + 1;
    let ny_offset = if let Ok(ny_offset) = u8::try_from(ny_offset) {
        ny_offset
    } else {
        debug_assert!(
            false,
            "Expected small new years offset, got {ny_offset} in ISO year {related_iso} (bounds {year_bounds:?})"
        );
        0
    };
    PackedChineseBasedYearInfo::new(month_lengths, leap_month, ny_offset)
}

#[cfg(feature = "datagen")]
pub(crate) fn compute_many_packed<CB: ChineseBased>(
    related_isos: core::ops::Range<i32>,
) -> alloc::vec::Vec<PackedChineseBasedYearInfo> {
    related_isos
        .map(|related_iso| {
            let mid_year = calendrical_calculations::iso::fixed_from_iso(related_iso, 7, 1);
            let year_bounds = YearBounds::compute::<CB>(mid_year);

            compute_packed_with_yb::<CB>(related_iso, year_bounds)
        })
        .collect()
}

impl<CB: ChineseBased> PrecomputedDataSource<ChineseBasedYearInfo>
    for ChineseBasedPrecomputedData<'_, CB>
{
    fn load_or_compute_info(&self, related_iso: i32) -> ChineseBasedYearInfo {
        self.data
            .and_then(|d| d.get_for_related_iso(related_iso))
            .unwrap_or_else(|| compute_cache::<CB>(related_iso))
    }
}

impl<'b, CB: ChineseBased> ChineseBasedPrecomputedData<'b, CB> {
    pub(crate) fn new(data: Option<&'b ChineseBasedCache<'b>>) -> Self {
        Self {
            data,
            _cb: PhantomData,
        }
    }
    /// Given an ISO date (in both ArithmeticDate and R.D. format), returns the ChineseBasedYearInfo and extended year for that date, loading
    /// from cache or computing.
    fn load_or_compute_info_for_iso(
        &self,
        rd: RataDie,
        iso: ArithmeticDate<Iso>,
    ) -> ChineseBasedYearInfo {
        if let Some(cached) = self.data.and_then(|d| d.get_for_iso_date(iso)) {
            return cached;
        };
        // compute

        let mid_year = calendrical_calculations::iso::fixed_from_iso(iso.year, 7, 1);
        let year_bounds = YearBounds::compute::<CB>(mid_year);
        let YearBounds { new_year, .. } = year_bounds;
        if rd >= new_year {
            compute_cache_with_yb::<CB>(iso.year, year_bounds)
        } else {
            compute_cache::<CB>(iso.year - 1)
        }
    }
}
/// A data struct used to load and use information for a set of ChineseBasedDates
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
// TODO(#3933): potentially make this smaller
pub(crate) struct ChineseBasedYearInfo {
    /// Contains:
    /// - length of each month in the year
    /// - whether or not there is a leap month, and which month it is
    /// - the date of Chinese New Year in the related ISO year
    packed_data: PackedChineseBasedYearInfo,
    pub(crate) related_iso: i32,
}

impl From<ChineseBasedYearInfo> for i32 {
    fn from(value: ChineseBasedYearInfo) -> Self {
        value.related_iso
    }
}

impl ChineseBasedYearInfo {
    pub(crate) fn new(related_iso: i32, packed_data: PackedChineseBasedYearInfo) -> Self {
        Self {
            packed_data,
            related_iso,
        }
    }

    /// Get the new year R.D. given the extended year that this yearinfo is for    
    pub(crate) fn new_year(self) -> RataDie {
        self.packed_data.ny_rd(self.related_iso)
    }

    /// Get the next new year R.D. given the extended year that this yearinfo is for
    /// (i.e, this year, not next year)
    fn next_new_year(self) -> RataDie {
        self.new_year() + i64::from(self.packed_data.days_in_year())
    }

    /// Get which month is the leap month. This produces the month *number*
    /// that is the leap month (not the ordinal month). In other words, for
    /// a year with an M05L, this will return Some(5). Note that the regular month precedes
    /// the leap month.
    pub(crate) fn leap_month(self) -> Option<NonZeroU8> {
        self.packed_data.leap_month_idx()
    }

    /// The last day of year in the previous month.
    /// `month` is 1-indexed, and the returned value is also
    /// a 1-indexed day of year
    ///
    /// Will be zero for the first month as the last day of the previous month
    /// is not in this year
    fn last_day_of_previous_month(self, month: u8) -> u16 {
        debug_assert!((1..=13).contains(&month), "Month out of bounds!");
        // Get the last day of the previous month.
        // Since `month` is 1-indexed, this needs to check if the month is 1 for the zero case
        if month == 1 {
            0
        } else {
            self.packed_data.last_day_of_month(month - 1)
        }
    }

    fn days_in_year(self) -> u16 {
        self.packed_data.days_in_year()
    }

    /// The last day of year in the current month.
    /// `month` is 1-indexed, and the returned value is also
    /// a 1-indexed day of year
    ///
    /// Will be zero for the first month as the last day of the previous month
    /// is not in this year
    fn last_day_of_month(self, month: u8) -> u16 {
        debug_assert!((1..=13).contains(&month), "Month out of bounds!");
        self.packed_data.last_day_of_month(month)
    }

    fn days_in_month(self, month: u8) -> u8 {
        let ret =
            u8::try_from(self.last_day_of_month(month) - self.last_day_of_previous_month(month));
        debug_assert!(ret.is_ok(), "Month too big!");
        ret.unwrap_or(30)
    }
}

impl<C: ChineseBasedWithDataLoading + CalendarArithmetic<YearInfo = ChineseBasedYearInfo>>
    ChineseBasedDateInner<C>
{
    /// Get a ChineseBasedDateInner from a RD and the cache/extended year associated with it
    fn chinese_based_date_from_info(
        date: RataDie,
        year: ChineseBasedYearInfo,
    ) -> ChineseBasedDateInner<C> {
        debug_assert!(
            date < year.next_new_year(),
            "Stored date {date:?} out of bounds!"
        );
        // 1-indexed day of year
        let day_of_year = u16::try_from(date - year.new_year() + 1);
        debug_assert!(day_of_year.is_ok(), "Somehow got a very large year in data");
        let day_of_year = day_of_year.unwrap_or(1);
        let mut month = 1;
        // TODO(#3933) perhaps use a binary search
        for iter_month in 1..=13 {
            month = iter_month;
            if year.last_day_of_month(iter_month) >= day_of_year {
                break;
            }
        }

        debug_assert!((1..=13).contains(&month), "Month out of bounds!");

        debug_assert!(
            month < 13 || year.leap_month().is_some(),
            "Cannot have 13 months in a non-leap year!"
        );
        let day_before_month_start = year.last_day_of_previous_month(month);
        let day_of_month = day_of_year - day_before_month_start;
        let day_of_month = u8::try_from(day_of_month);
        debug_assert!(day_of_month.is_ok(), "Month too big!");
        let day_of_month = day_of_month.unwrap_or(1);

        // This can use `new_unchecked` because this function is only ever called from functions which
        // generate the year, month, and day; therefore, there should never be a situation where
        // creating this ArithmeticDate would fail, since the same algorithms used to generate the ymd
        // are also used to check for valid ymd.
        ChineseBasedDateInner(ArithmeticDate::new_unchecked(year, month, day_of_month))
    }

    /// Get a ChineseBasedDateInner from a RD, with the related ISO date
    /// (passed in to avoid recomputing)
    pub(crate) fn chinese_based_date_from_rd(
        cal: &C,
        rd: RataDie,
        iso: ArithmeticDate<Iso>,
    ) -> ChineseBasedDateInner<C> {
        let data = cal.get_precomputed_data();

        let year = data.load_or_compute_info_for_iso(rd, iso);

        Self::chinese_based_date_from_info(rd, year)
    }

    pub(crate) fn new_year(self) -> RataDie {
        self.0.year.new_year()
    }

    /// Get a RataDie from a ChineseBasedDateInner
    ///
    /// This finds the RataDie of the new year of the year given, then finds the RataDie of the new moon
    /// (beginning of the month) of the month given, then adds the necessary number of days.
    pub(crate) fn rd_from_chinese_based_date_inner(date: ChineseBasedDateInner<C>) -> RataDie {
        let first_day_of_year = date.new_year();
        let day_of_year = date.day_of_year(); // 1 indexed
        first_day_of_year + i64::from(day_of_year) - 1
    }

    /// Create a new arithmetic date from a year, month ordinal, and day with bounds checking; returns the
    /// result of creating this arithmetic date, as well as a ChineseBasedYearInfo - either the one passed in
    /// optionally as an argument, or a new ChineseBasedYearInfo for the given year, month, and day args.
    pub(crate) fn new_from_ordinals(
        year: ChineseBasedYearInfo,
        month: u8,
        day: u8,
    ) -> Result<ArithmeticDate<C>, DateError> {
        let max_month = Self::months_in_year(year);
        if month == 0 || !(1..=max_month).contains(&month) {
            return Err(DateError::Range {
                field: "month",
                value: month as i32,
                min: 1,
                max: max_month as i32,
            });
        }

        let max_day = year.days_in_month(month);
        if day == 0 || day > max_day {
            return Err(DateError::Range {
                field: "day",
                value: day as i32,
                min: 1,
                max: max_day as i32,
            });
        }

        // Unchecked can be used because month and day are already checked in this fn
        Ok(ArithmeticDate::<C>::new_unchecked(year, month, day))
    }

    /// Call `months_in_year` on a `ChineseBasedDateInner`
    pub(crate) fn months_in_year_inner(&self) -> u8 {
        Self::months_in_year(self.0.year)
    }

    /// Return the number of months in a given year, which is 13 in a leap year, and 12 in a common year.
    /// Also takes a `ChineseBasedYearInfo` argument.
    fn months_in_year(year: ChineseBasedYearInfo) -> u8 {
        if year.leap_month().is_some() {
            13
        } else {
            12
        }
    }

    /// Calls `days_in_month` on an instance of ChineseBasedDateInner
    pub(crate) fn days_in_month_inner(&self) -> u8 {
        self.0.year.days_in_month(self.0.month)
    }

    /// Calls days_in_year on an instance of ChineseBasedDateInner
    pub(crate) fn days_in_year_inner(&self) -> u16 {
        self.0.year.days_in_year()
    }

    /// Calculate the number of days in the year so far for a ChineseBasedDate;
    /// similar to `CalendarArithmetic::day_of_year`
    pub(crate) fn day_of_year(&self) -> u16 {
        self.0.year.last_day_of_previous_month(self.0.month) + u16::from(self.0.day)
    }

    /// The calendar-specific month code represented by `date`;
    /// since the Chinese calendar has leap months, an "L" is appended to the month code for
    /// leap months. For example, in a year where an intercalary month is added after the second
    /// month, the month codes for ordinal months 1, 2, 3, 4, 5 would be "M01", "M02", "M02L", "M03", "M04".
    pub(crate) fn month(&self) -> MonthInfo {
        let ordinal = self.0.month;
        let leap_month_option = self.0.year.leap_month();

        // 1 indexed leap month name. This is also the ordinal for the leap month
        // in the year (e.g. in `M01, M01L, M02, ..`, the leap month is for month 1, and it is also
        // ordinally `month 2`, zero-indexed)
        let leap_month = if let Some(leap) = leap_month_option {
            leap.get()
        } else {
            // sentinel value
            14
        };
        let code_inner = if leap_month == ordinal {
            // Month cannot be 1 because a year cannot have a leap month before the first actual month,
            // and the maximum num of months ina leap year is 13.
            debug_assert!((2..=13).contains(&ordinal));
            match ordinal {
                2 => tinystr!(4, "M01L"),
                3 => tinystr!(4, "M02L"),
                4 => tinystr!(4, "M03L"),
                5 => tinystr!(4, "M04L"),
                6 => tinystr!(4, "M05L"),
                7 => tinystr!(4, "M06L"),
                8 => tinystr!(4, "M07L"),
                9 => tinystr!(4, "M08L"),
                10 => tinystr!(4, "M09L"),
                11 => tinystr!(4, "M10L"),
                12 => tinystr!(4, "M11L"),
                13 => tinystr!(4, "M12L"),
                _ => tinystr!(4, "und"),
            }
        } else {
            let mut adjusted_ordinal = ordinal;
            if ordinal > leap_month {
                // Before adjusting for leap month, if ordinal > leap_month,
                // the month cannot be 1 because this implies the leap month is < 1, which is impossible;
                // cannot be 2 because that implies the leap month is = 1, which is impossible,
                // and cannot be more than 13 because max number of months in a year is 13.
                debug_assert!((2..=13).contains(&ordinal));
                adjusted_ordinal -= 1;
            }
            debug_assert!((1..=12).contains(&adjusted_ordinal));
            match adjusted_ordinal {
                1 => tinystr!(4, "M01"),
                2 => tinystr!(4, "M02"),
                3 => tinystr!(4, "M03"),
                4 => tinystr!(4, "M04"),
                5 => tinystr!(4, "M05"),
                6 => tinystr!(4, "M06"),
                7 => tinystr!(4, "M07"),
                8 => tinystr!(4, "M08"),
                9 => tinystr!(4, "M09"),
                10 => tinystr!(4, "M10"),
                11 => tinystr!(4, "M11"),
                12 => tinystr!(4, "M12"),
                _ => tinystr!(4, "und"),
            }
        };
        let code = MonthCode(code_inner);
        MonthInfo {
            ordinal,
            standard_code: code,
            formatting_code: code,
        }
    }
}

impl<C: ChineseBasedWithDataLoading> CalendarArithmetic for C {
    type YearInfo = ChineseBasedYearInfo;

    fn days_in_provided_month(year: ChineseBasedYearInfo, month: u8) -> u8 {
        year.days_in_month(month)
    }

    /// Returns the number of months in a given year, which is 13 in a leap year, and 12 in a common year.
    fn months_in_provided_year(year: ChineseBasedYearInfo) -> u8 {
        if year.leap_month().is_some() {
            13
        } else {
            12
        }
    }

    /// Returns true if the given year is a leap year, and false if not.
    fn provided_year_is_leap(year: ChineseBasedYearInfo) -> bool {
        year.leap_month().is_some()
    }

    /// Returns the (month, day) of the last day in a Chinese year (the day before Chinese New Year).
    /// The last month in a year will always be 12 in a common year or 13 in a leap year. The day is
    /// determined by finding the day immediately before the next new year and calculating the number
    /// of days since the last new moon (beginning of the last month in the year).
    fn last_month_day_in_provided_year(year: ChineseBasedYearInfo) -> (u8, u8) {
        if year.leap_month().is_some() {
            (13, year.days_in_month(13))
        } else {
            (12, year.days_in_month(12))
        }
    }

    fn days_in_provided_year(year: ChineseBasedYearInfo) -> u16 {
        year.last_day_of_month(13)
    }
}

/// Get the ordinal lunar month from a code for chinese-based calendars.
pub(crate) fn chinese_based_ordinal_lunar_month_from_code(
    code: MonthCode,
    year: ChineseBasedYearInfo,
) -> Option<u8> {
    let leap_month = if let Some(leap) = year.leap_month() {
        leap.get()
    } else {
        // 14 is a sentinel value, greater than all other months, for the purpose of computation only;
        // it is impossible to actually have 14 months in a year.
        14
    };

    if code.0.len() < 3 {
        return None;
    }
    let bytes = code.0.all_bytes();
    if bytes[0] != b'M' {
        return None;
    }
    if code.0.len() == 4 && bytes[3] != b'L' {
        return None;
    }
    // Unadjusted is zero-indexed month index, must add one to it to use
    let mut unadjusted = 0;
    if bytes[1] == b'0' {
        if bytes[2] >= b'1' && bytes[2] <= b'9' {
            unadjusted = bytes[2] - b'0';
        }
    } else if bytes[1] == b'1' && bytes[2] >= b'0' && bytes[2] <= b'2' {
        unadjusted = 10 + bytes[2] - b'0';
    }
    if bytes[3] == b'L' {
        // Asked for a leap month that doesn't exist
        if unadjusted + 1 != leap_month {
            return None;
        } else {
            // The leap month occurs after the regular month of the same name
            return Some(unadjusted + 1);
        }
    }
    if unadjusted != 0 {
        // If the month has an index greater than that of the leap month,
        // bump it up by one
        if unadjusted + 1 > leap_month {
            return Some(unadjusted + 1);
        } else {
            return Some(unadjusted);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    fn packed_roundtrip_single(
        mut month_lengths: [bool; 13],
        leap_month_idx: Option<NonZeroU8>,
        ny_offset: u8,
    ) {
        if leap_month_idx.is_none() {
            // Avoid bad invariants
            month_lengths[12] = false;
        }
        let packed = PackedChineseBasedYearInfo::new(month_lengths, leap_month_idx, ny_offset);

        assert_eq!(
            ny_offset,
            packed.ny_offset(),
            "Roundtrip with {month_lengths:?}, {leap_month_idx:?}, {ny_offset}"
        );
        assert_eq!(
            leap_month_idx,
            packed.leap_month_idx(),
            "Roundtrip with {month_lengths:?}, {leap_month_idx:?}, {ny_offset}"
        );
        let mut month_lengths_roundtrip = [false; 13];
        for (i, len) in month_lengths_roundtrip.iter_mut().enumerate() {
            *len = packed.month_has_30_days(i as u8 + 1);
        }
        assert_eq!(
            month_lengths, month_lengths_roundtrip,
            "Roundtrip with {month_lengths:?}, {leap_month_idx:?}, {ny_offset}"
        );
    }

    #[test]
    fn test_roundtrip_packed() {
        const SHORT: [bool; 13] = [false; 13];
        const LONG: [bool; 13] = [true; 13];
        const ALTERNATING1: [bool; 13] = [
            false, true, false, true, false, true, false, true, false, true, false, true, false,
        ];
        const ALTERNATING2: [bool; 13] = [
            true, false, true, false, true, false, true, false, true, false, true, false, true,
        ];
        const RANDOM1: [bool; 13] = [
            true, true, false, false, true, true, false, true, true, true, true, false, true,
        ];
        const RANDOM2: [bool; 13] = [
            false, true, true, true, true, false, true, true, true, false, false, true, false,
        ];
        packed_roundtrip_single(SHORT, None, 5);
        packed_roundtrip_single(SHORT, None, 10);
        packed_roundtrip_single(SHORT, NonZeroU8::new(11), 15);
        packed_roundtrip_single(LONG, NonZeroU8::new(12), 15);
        packed_roundtrip_single(ALTERNATING1, None, 2);
        packed_roundtrip_single(ALTERNATING1, NonZeroU8::new(3), 5);
        packed_roundtrip_single(ALTERNATING2, None, 9);
        packed_roundtrip_single(ALTERNATING2, NonZeroU8::new(7), 26);
        packed_roundtrip_single(RANDOM1, None, 29);
        packed_roundtrip_single(RANDOM1, NonZeroU8::new(12), 29);
        packed_roundtrip_single(RANDOM1, NonZeroU8::new(2), 21);
        packed_roundtrip_single(RANDOM2, None, 25);
        packed_roundtrip_single(RANDOM2, NonZeroU8::new(2), 19);
        packed_roundtrip_single(RANDOM2, NonZeroU8::new(5), 2);
        packed_roundtrip_single(RANDOM2, NonZeroU8::new(12), 5);
    }
}
