use image::RgbImage;
use rusttype::Font;

use crate::{
	colours,
	common_types::Range,
	drawing::{
		draw_graph_bars, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_bar_labels, MarkIntervals, Padding, Spacing,
	},
	from_args::{data_from_args, FromArgs},
	util::next_multiple,
};

const PADDING: Padding = Padding {
	above: 7,
	below: 19,
	left: 21,
	right: 3,
};
const SPACING: Spacing = Spacing {
	horizontal: 8,
	vertical: 16,
};
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &Font, data: Vec<HourlyPrecipitation>) -> RgbImage {
	let max_chart_precipitation = next_multiple(
		data.iter()
			.flat_map(|hour| [hour.rain as i32, hour.snow as i32])
			.max()
			.unwrap_or(0),
		1,
	) as u32;
	let width = data.len() as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_chart_precipitation * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, colours::BACKGROUND);
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_bar_labels(
		&mut canvas,
		data.iter().map(|hour| hour.hour),
		MarkIntervals::new(1, 2),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_chart_precipitation as i32),
		MarkIntervals::new(1, 1),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	draw_graph_bars(
		&mut canvas,
		data.iter().map(|hour| hour.rain as i32),
		colours::RAIN,
		PADDING,
		SPACING,
	);
	draw_graph_bars(
		&mut canvas,
		data.iter().map(|hour| hour.snow as i32),
		colours::SNOW,
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
pub struct HourlyPrecipitation {
	/// Hour of the day
	hour: u8,
	/// Amount of rain in mm * 100
	rain: u32,
	/// Amount of snow in mm * 100
	snow: u32,
}

impl FromArgs<3> for HourlyPrecipitation {
	fn from_args([hour, rain, snow]: [String; 3]) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let rain = rain.parse().expect("Could not parse a rain argument");
		let snow = snow.parse().expect("Could not parse a snow argument");
		Self { hour, rain, snow }
	}
}
