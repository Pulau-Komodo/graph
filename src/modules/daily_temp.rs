use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	colours,
	common_types::{Point, Range},
	drawing::{
		draw_line_segment, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, Padding,
	},
};

const SPACE_ABOVE: u32 = 7;
const SPACE_BELOW: u32 = 19;
const SPACE_LEFT: u32 = 21;
const SPACE_RIGHT: u32 = 9;
const PIXELS_PER_CELSIUS: u32 = 3;
const PIXELS_PER_DAY: u32 = 25;
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	let temp_range = calculate_grid_range(&data);
	let width = PIXELS_PER_DAY * (data.len() - 1) as u32 + SPACE_LEFT + SPACE_RIGHT;
	let height = temp_range.len() as u32 * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE + SPACE_BELOW;
	const PADDING: Padding = Padding {
		above: SPACE_ABOVE,
		below: SPACE_BELOW,
		left: SPACE_LEFT,
		right: SPACE_RIGHT,
	};
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, Rgb::<u8>([0, 0, 0]));
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|day| day.day),
		1,
		1,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_DAY,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		temp_range,
		2,
		4,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_CELSIUS,
	);
	draw_temp_lines(&mut canvas, &data, temp_range.end(), false);
	draw_temp_lines(&mut canvas, &data, temp_range.end(), true);
	canvas
}

#[derive(Debug, Clone, Copy)]
struct DayData {
	/// Day of the month
	day: u8,
	/// Minimum temperature in centidegrees Celsius
	temp_min: i32,
	/// Maximum temperature in centidegrees Celsius
	temp_max: i32,
}

impl DayData {
	fn from_args(day: String, temp_min: String, temp_max: String) -> Self {
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

fn data_from_args(args: Vec<String>) -> Vec<DayData> {
	let mut data = Vec::with_capacity(args.len() / 3);
	for mut item in args.into_iter().chunks(3).into_iter() {
		let (day, temp_min, temp_max) = item
			.next_tuple()
			.expect("Arguments could not be grouped into threes");
		let day_data = DayData::from_args(day, temp_min, temp_max);
		data.push(day_data);
	}
	data
}

/// Get the lowest and highest temperatures that the grid will show. These are the closest multiples of 4 degrees Celsius that include all data.
fn calculate_grid_range(data: &[DayData]) -> Range<i32> {
	let (all_temps_min, all_temps_max) = data.iter().fold(
		(i32::MAX, i32::MIN),
		|(min, max),
		 DayData {
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

/// Draws the temperature lines onto the canvas. If max is true it draws the maximum temperature lines, otherwise it draws the minimum temperature lines.
fn draw_temp_lines(canvas: &mut RgbImage, data: &[DayData], temp_range_max: i32, max: bool) {
	let colour = if max {
		colours::MAX_TEMP
	} else {
		colours::MIN_TEMP
	};
	for (index, (start, end)) in data.iter().tuple_windows().enumerate() {
		let start_temp = if max { start.temp_max } else { start.temp_min };
		let end_temp = if max { end.temp_max } else { end.temp_min };
		let start = Point {
			x: index as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: start_temp.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		let end = Point {
			x: (index + 1) as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: end_temp.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		draw_line_segment(canvas, start, end, colour);
	}
}
