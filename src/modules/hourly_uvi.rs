use ab_glyph::FontRef;
use image::RgbImage;
use itertools::Itertools;

use crate::{
	colours,
	common_types::{GradientPoint, MultiPointGradient, Range},
	drawing::{
		draw_graph_bars_with_gradient, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
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
	vertical: 10,
};
const FONT_SCALE: ab_glyph::PxScale = ab_glyph::PxScale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &FontRef, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &FontRef, data: Vec<HourlyUvi>) -> RgbImage {
	let max_chart_uvi = next_multiple(
		data.iter().map(|hour| hour.uvi).max().unwrap_or(0) as i32,
		1,
	);
	let width = data.len() as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_chart_uvi as u32 * SPACING.vertical / 100 + PADDING.vertical();
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
		Range::new(0, max_chart_uvi),
		MarkIntervals::new(1, 1),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	let gradient = MultiPointGradient::new(vec![
		GradientPoint::from_rgb(PADDING.below, colours::UVI_LOW),
		GradientPoint::from_rgb(
			PADDING.below + SPACING.vertical * 9 / 2,
			colours::UVI_MEDIUM,
		),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 9, colours::UVI_HIGH),
	]);
	draw_graph_bars_with_gradient(
		&mut canvas,
		data.iter().map(|day| day.uvi as i32),
		gradient,
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
pub struct HourlyUvi {
	/// Hour of the day
	hour: u8,
	/// UV index * 100
	uvi: u16,
}

impl HourlyUvi {
	fn from_args(hour: String, uvi: String) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let uvi = uvi.parse().expect("Could not parse a UV index argument");
		Self { hour, uvi }
	}
}

fn data_from_args(args: Vec<String>) -> Vec<HourlyUvi> {
	const CHUNK_SIZE: usize = 2;
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let (hour, uvi) = item
			.next_tuple()
			.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"));
		let uvi_datum = HourlyUvi::from_args(hour, uvi);
		data.push(uvi_datum);
	}
	data
}
