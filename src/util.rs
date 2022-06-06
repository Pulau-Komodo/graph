use crate::common_types::Range;

/// The highest value the chart will include.
pub fn next_multiple(highest: i32, interval: i32) -> i32 {
	let interval = interval * 100;
	let round_up = match highest.rem_euclid(interval) {
		0 => 0,
		n => interval - n,
	};
	highest + round_up
}

/// Get the lowest and highest values that the chart will include.
pub fn previous_and_next_multiple(range: Range<i32>, interval: i32) -> Range<i32> {
	let interval = interval * 100;
	let round_up = match range.end().rem_euclid(interval) {
		0 => 0,
		n => interval - n,
	};
	Range::new(
		range.start() - range.start().rem_euclid(interval),
		range.end() + round_up,
	)
}
