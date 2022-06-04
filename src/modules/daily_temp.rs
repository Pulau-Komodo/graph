use image::{Rgb, RgbImage};
use rusttype::Font;

use crate::{
	colours,
	common_types::Range,
	drawing::{
		draw_graph_lines, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
	from_args::{data_from_args, FromArgs},
};
const PADDING: Padding = Padding {
	above: 7,
	below: 19,
	left: 21,
	right: 9,
};
const SPACING: Spacing = Spacing {
	horizontal: 3,
	vertical: 25,
};
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data: Vec<HourlyTempData> = data_from_args(args);
	let temp_range = calculate_grid_range(&data);
	let width = SPACING.horizontal * (data.len() - 1) as u32 + PADDING.horizontal();
	let height = temp_range.len() as u32 * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, Rgb::<u8>([0, 0, 0]));
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|day| day.day),
		MarkIntervals::new(1, 1),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		temp_range,
		MarkIntervals::new(2, 4),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|daily| daily.temp_min),
		colours::MIN_TEMP,
		temp_range.end(),
		PADDING,
		SPACING,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|daily| daily.temp_max),
		colours::MAX_TEMP,
		temp_range.end(),
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
struct HourlyTempData {
	/// Day of the month
	day: u8,
	/// Minimum temperature in centidegrees Celsius
	temp_min: i32,
	/// Maximum temperature in centidegrees Celsius
	temp_max: i32,
}

impl FromArgs<3> for HourlyTempData {
	fn from_args([day, temp_min, temp_max]: [String; 3]) -> Self {
		let day = day.parse().expect("Could not parse a day argument");
		let temp_min = temp_min
			.parse()
			.expect("Could not parse a minimum temperature argument");
		let temp_max = temp_max
			.parse()
			.expect("Could not parse a maximum temperature argument");
		Self {
			day,
			temp_min,
			temp_max,
		}
	}
}

/// Get the lowest and highest temperatures that the grid will show. These are the closest multiples of 4 degrees Celsius that include all data.
fn calculate_grid_range(data: &[HourlyTempData]) -> Range<i32> {
	let (all_temps_min, all_temps_max) = data.iter().fold(
		(i32::MAX, i32::MIN),
		|(min, max),
		 HourlyTempData {
		     temp_min, temp_max, ..
		 }| { (min.min(*temp_min), max.max(*temp_max)) },
	);
	let round_up = match all_temps_max.rem_euclid(400) {
		0 => 0,
		n => 400 - n,
	};
	Range::new(
		all_temps_min - all_temps_min.rem_euclid(400),
		all_temps_max + round_up,
	)
}
