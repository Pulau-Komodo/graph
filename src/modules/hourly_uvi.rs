use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	common_types::{GradientPoint, MultiPointGradient, Range},
	drawing::{
		draw_graph_lines_with_gradient, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
		vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
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
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	let max_grid_uvi = calculate_max_grid_uvi(&data);
	let width = (data.len() - 1) as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_grid_uvi as u32 * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, Rgb::<u8>([0, 0, 0]));
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|datum| datum.hour),
		MarkIntervals::new(1, 2),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_grid_uvi as i32),
		MarkIntervals::new(1, 1),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	let gradient = MultiPointGradient::new(vec![
		GradientPoint::from_rgb(PADDING.below, [0, 255, 33]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 9 / 2, [255, 255, 33]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 9, [255, 0, 33]),
	]);
	draw_graph_lines_with_gradient(
		&mut canvas,
		data.iter().map(|day| day.uvi as i32),
		gradient,
		max_grid_uvi as i32,
		PADDING,
		SPACING,
	);
	canvas
}

#[derive(Debug, Clone, Copy)]
struct HourlyUviDatum {
	/// Hour of the day
	hour: u8,
	/// UV index * 100
	uvi: u16,
}

impl HourlyUviDatum {
	fn from_args(hour: String, uvi: String) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let uvi = uvi.parse().expect("Could not parse a UV index argument");
		Self { hour, uvi }
	}
}

fn data_from_args(args: Vec<String>) -> Vec<HourlyUviDatum> {
	const CHUNK_SIZE: usize = 2;
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let (hour, uvi) = item
			.next_tuple()
			.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"));
		let uvi_datum = HourlyUviDatum::from_args(hour, uvi);
		data.push(uvi_datum);
	}
	data
}

/// The highest centi-UVI the grid will display.
fn calculate_max_grid_uvi(data: &[HourlyUviDatum]) -> u16 {
	let highest_uv = data.iter().fold(u16::MIN, |acc, datum| acc.max(datum.uvi));
	let round_up = match highest_uv.rem_euclid(100) {
		0 => 0,
		n => 100 - n,
	};
	highest_uv + round_up
}
