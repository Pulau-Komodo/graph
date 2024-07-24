use ab_glyph::FontRef;
use image::RgbImage;
use itertools::Itertools;

use crate::{
	colours,
	common_types::Range,
	drawing::{
		draw_graph_bars, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
	util::next_multiple,
};

const PADDING: Padding = Padding {
	above: 7,
	below: 19,
	left: 21,
	right: 19,
};
const SPACING: Spacing = Spacing {
	horizontal: 6,
	vertical: 16,
};
const FONT_SCALE: ab_glyph::PxScale = ab_glyph::PxScale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &FontRef, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &FontRef, data: Vec<MinutelyPrecipitation>) -> RgbImage {
	let max_chart_precipitation = next_multiple(
		data.iter()
			.map(|minute| minute.precipitation as i32)
			.max()
			.unwrap_or(0),
		1,
	) as u32;
	let width = data.len() as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_chart_precipitation * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, colours::BACKGROUND);
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|minute| minute.minute),
		MarkIntervals::new(3, 3),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
		true,
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
		data.into_iter()
			.map(|minutely| minutely.precipitation as i32),
		colours::RAIN,
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
pub struct MinutelyPrecipitation {
	/// Minute of the hour
	minute: u8,
	/// Precipitation in mm / h * 100
	precipitation: u16,
}

impl MinutelyPrecipitation {
	fn from_args(minute: String, precipitation: String) -> Self {
		let minute = minute.parse().expect("Could not parse a minute argument");
		let precipitation = precipitation
			.parse()
			.expect("Could not parse a precipitation argument");
		Self {
			minute,
			precipitation,
		}
	}
}

fn data_from_args(args: Vec<String>) -> Vec<MinutelyPrecipitation> {
	const CHUNK_SIZE: usize = 2;
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let (minute, precipitation) = item
			.next_tuple()
			.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"));
		let precipitation_datum = MinutelyPrecipitation::from_args(minute, precipitation);
		data.push(precipitation_datum);
	}
	data
}
