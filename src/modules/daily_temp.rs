use image::RgbImage;
use itertools::Itertools;
use rusttype::Font;

use crate::{
	colours,
	common_types::Range,
	drawing::{
		draw_graph_lines, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
	from_args::{data_from_args, FromArgs},
	util::previous_and_next_multiple,
};
const PADDING: Padding = Padding {
	above: 7,
	below: 19,
	left: 21,
	right: 9,
};
const SPACING: Spacing = Spacing {
	horizontal: 25,
	vertical: 3,
};
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

/// Makes a graph showing daily min and max temp.
///
/// Arguments are in the format day, temp min, temp max, repeat. Temperatures are in centidegrees Celsius.
///
/// Example input values: `28 -555 -333 29 -222 111 30 -333 222 1 0 444 2 222 555 3 111 666 4 222 555 5 555 2222`.
pub fn parse_and_create(font: &Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &Font, data: Vec<HourlyTemps>) -> RgbImage {
	let temp_range = data
		.iter()
		.flat_map(|day| [day.temp_min, day.temp_max])
		.minmax()
		.into_option()
		.unwrap_or((0, 0));
	let chart_temp_range = previous_and_next_multiple(Range::new(temp_range.0, temp_range.1), 4);
	let width = SPACING.horizontal * (data.len() - 1) as u32 + PADDING.horizontal();
	let height = chart_temp_range.len() as u32 * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, colours::BACKGROUND);
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|day| day.day),
		MarkIntervals::new(1, 1),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		chart_temp_range,
		MarkIntervals::new(2, 4),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|daily| daily.temp_min),
		colours::MIN_TEMP,
		chart_temp_range.end(),
		PADDING,
		SPACING,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|daily| daily.temp_max),
		colours::MAX_TEMP,
		chart_temp_range.end(),
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
pub struct HourlyTemps {
	/// Day of the month
	day: u8,
	/// Minimum temperature in centidegrees Celsius
	temp_min: i32,
	/// Maximum temperature in centidegrees Celsius
	temp_max: i32,
}

impl FromArgs<3> for HourlyTemps {
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
