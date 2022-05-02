use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	colours,
	common_types::{Point, Range},
	drawing::{draw_line_segment, draw_outer_lines, fill_canvas, Padding},
};

const SPACE_ABOVE: u32 = 7;
const SPACE_BELOW: u32 = 19;
const SPACE_LEFT: u32 = 21;
const SPACE_RIGHT: u32 = 9;
const PIXELS_PER_CELSIUS: u32 = 3;
const PIXELS_PER_DAY: u32 = 25;
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data = day_data_from_args(args);
	let temp_range = calculate_grid_range(&data);
	let width = PIXELS_PER_DAY * (data.len() - 1) as u32 + SPACE_LEFT + SPACE_RIGHT;
	let height = temp_range.len() as u32 * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE + SPACE_BELOW;
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, Rgb::<u8>([0, 0, 0]));
	draw_outer_lines(
		&mut canvas,
		Padding {
			above: SPACE_ABOVE,
			below: SPACE_BELOW,
			left: SPACE_LEFT,
			right: SPACE_RIGHT,
		},
	);
	day_lines_and_labels(&mut canvas, &data, &font);
	temp_lines_and_labels(&mut canvas, temp_range, &font);
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

fn day_data_from_args(args: Vec<String>) -> Vec<DayData> {
	let mut pairs = Vec::with_capacity(args.len() / 3);
	for mut item in args.into_iter().chunks(3).into_iter() {
		let (day, temp_min, temp_max) = item
			.next_tuple()
			.expect("Arguments could not be grouped into threes");
		let data = DayData::from_args(day, temp_min, temp_max);
		pairs.push(data);
	}
	pairs
}

/// Get the lowest and highest temperatures that the grid will show. These are the closest multiples of 4 degrees Celsius that include all data.
fn calculate_grid_range(data: &[DayData]) -> Range<i32> {
	let (temp_range_min, temp_range_max) = {
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
		(
			all_temps_min - all_temps_min.rem_euclid(400),
			all_temps_max + round_up,
		)
	};
	Range::new(temp_range_min, temp_range_max)
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

fn temp_lines_and_labels(canvas: &mut RgbImage, temp_range: Range<i32>, font: &rusttype::Font) {
	let width = canvas.width();
	let max_temp = temp_range.end() / 100;
	for temp in (temp_range.start()..temp_range.end())
		.step_by(200)
		.map(|n| n / 100)
	{
		let y = SPACE_ABOVE + max_temp.abs_diff(temp) * PIXELS_PER_CELSIUS;
		let line_colour = if temp == 0 {
			colours::MAIN_LINES
		} else {
			colours::GRID_LINES
		};
		draw_line_segment(
			canvas,
			Point { x: SPACE_LEFT, y },
			Point {
				x: width - SPACE_RIGHT,
				y,
			},
			line_colour,
		);
		if temp % 4 == 0 {
			let text = &format!("{}", temp);
			let (text_width, text_height) = imageproc::drawing::text_size(FONT_SCALE, font, text);
			imageproc::drawing::draw_text_mut(
				canvas,
				colours::TEXT,
				SPACE_LEFT as i32 - text_width - 3,
				y as i32 - text_height / 2,
				FONT_SCALE,
				font,
				text,
			);
		}
	}
}

fn day_lines_and_labels(canvas: &mut RgbImage, data: &[DayData], font: &rusttype::Font) {
	let height = canvas.height();
	for (index, day) in data.iter().enumerate() {
		let x = SPACE_LEFT + index as u32 * PIXELS_PER_DAY;
		draw_line_segment(
			canvas,
			Point { x, y: SPACE_ABOVE },
			Point {
				x,
				y: height - SPACE_BELOW,
			},
			colours::GRID_LINES,
		);
		let text = &format!("{}", day.day);
		let (text_width, _text_height) = imageproc::drawing::text_size(FONT_SCALE, font, text);
		imageproc::drawing::draw_text_mut(
			canvas,
			colours::TEXT,
			x as i32 - text_width / 2,
			(height - SPACE_BELOW + 5) as i32,
			FONT_SCALE,
			font,
			text,
		);
	}
}
