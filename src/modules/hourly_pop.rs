use image::{Rgb, RgbImage};
use rusttype::Font;

use crate::{
	colours,
	common_types::Range,
	drawing::{
		draw_graph_bars, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
	from_args::{data_from_args, FromArgs},
};

const PADDING: Padding = Padding {
	above: 7,
	below: 19,
	left: 21,
	right: 3,
};
const SPACING: Spacing = Spacing {
	horizontal: 8,
	vertical: 1,
};
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &Font, data: Vec<HourlyPop>) -> RgbImage {
	let max_chart_pop = 10_000;
	let width = data.len() as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_chart_pop * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, colours::BACKGROUND);
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|datum| datum.hour),
		MarkIntervals::new(1, 2),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_chart_pop as i32),
		MarkIntervals::new(10, 20),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	draw_graph_bars(
		&mut canvas,
		data.iter().map(|datum| datum.chance as i32),
		Rgb([0, 148, 255]),
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
pub struct HourlyPop {
	/// Hour of the day
	hour: u8,
	/// Probability of precipitation * 100
	chance: u32,
}

impl FromArgs<2> for HourlyPop {
	fn from_args([hour, chance]: [String; 2]) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let chance = chance
			.parse::<u32>()
			.expect("Could not parse a probability of precipitation argument")
			* 100;
		Self { hour, chance }
	}
}
