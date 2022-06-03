use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	common_types::{GradientPoint, MultiPointGradient, Point, Range},
	drawing::{
		draw_line_segment_with_gradient, draw_outer_lines, fill_canvas,
		horizontal_lines_and_labels, vertical_lines_and_labels, Padding,
	},
};

const SPACE_ABOVE: u32 = 7;
const SPACE_BELOW: u32 = 19;
const SPACE_LEFT: u32 = 21;
const SPACE_RIGHT: u32 = 3;
const PIXELS_PER_UVI: u32 = 10;
const PIXELS_PER_HOUR: u32 = 8;
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	let max_grid_uvi = calculate_max_grid_uvi(&data);
	let width = (data.len() - 1) as u32 * PIXELS_PER_HOUR + SPACE_LEFT + SPACE_RIGHT;
	let height = max_grid_uvi as u32 * PIXELS_PER_UVI / 100 + SPACE_ABOVE + SPACE_BELOW;
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
		data.iter().map(|datum| datum.hour),
		1,
		2,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_HOUR,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_grid_uvi as i32),
		1,
		1,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_UVI,
	);
	draw_uvi_lines(&mut canvas, &data, max_grid_uvi);
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

/// Draws the UVI lines onto the canvas.
fn draw_uvi_lines(canvas: &mut RgbImage, data: &[HourlyUviDatum], grid_max_uvi: u16) {
	let gradient = MultiPointGradient::new(vec![
		GradientPoint::from_rgb(SPACE_BELOW, [0, 255, 33]),
		GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_UVI * 9 / 2, [255, 255, 33]),
		GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_UVI * 9, [255, 0, 33]),
	]);
	for (index, (start, end)) in data.iter().tuple_windows().enumerate() {
		let start = Point {
			x: index as u32 * PIXELS_PER_HOUR + SPACE_LEFT,
			y: start.uvi.abs_diff(grid_max_uvi) as u32 * PIXELS_PER_UVI / 100 + SPACE_ABOVE,
		};
		let end = Point {
			x: (index + 1) as u32 * PIXELS_PER_HOUR + SPACE_LEFT,
			y: end.uvi.abs_diff(grid_max_uvi) as u32 * PIXELS_PER_UVI / 100 + SPACE_ABOVE,
		};
		draw_line_segment_with_gradient(canvas, start, end, &gradient);
	}
}
