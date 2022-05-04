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
const SPACE_RIGHT: u32 = 9;
const PIXELS_PER_M_PER_S: u32 = 5;
const PIXELS_PER_HOUR: u32 = 8;
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	let max_grid_speed = calculate_max_grid_wind_speed(&data);
	let width = (data.len() - 1) as u32 * PIXELS_PER_HOUR + SPACE_LEFT + SPACE_RIGHT;
	let height = max_grid_speed as u32 * PIXELS_PER_M_PER_S / 100 + SPACE_ABOVE + SPACE_BELOW;
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
		data.iter().map(|data| data.hour),
		1,
		2,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_HOUR,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_grid_speed as i32),
		5,
		5,
		&font,
		FONT_SCALE,
		PADDING,
		PIXELS_PER_M_PER_S,
	);
	draw_wind_speed_lines(&mut canvas, &data, max_grid_speed, true);
	draw_wind_speed_lines(&mut canvas, &data, max_grid_speed, false);
	canvas
}

struct HourlyWindData {
	/// Hour of the day
	hour: u8,
	/// Wind speed in cm/s
	wind_speed: u16,
	/// Wind gust speed in cm/s
	wind_gust: u16,
	/// Wind direction in degrees, where 0 is north and 90 is east
	wind_direction: u16,
}

impl HourlyWindData {
	fn from_args(
		hour: String,
		wind_speed: String,
		wind_gust: String,
		wind_direction: String,
	) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let wind_speed = wind_speed
			.parse()
			.expect("Could not parse a wind speed argument");
		let wind_gust = wind_gust
			.parse()
			.expect("Could not parse a wind gust argument");
		let wind_direction = wind_direction
			.parse()
			.expect("Could not parse a wind direction argument");
		Self {
			hour,
			wind_speed,
			wind_gust,
			wind_direction,
		}
	}
}

fn data_from_args(args: Vec<String>) -> Vec<HourlyWindData> {
	const CHUNK_SIZE: usize = 4;
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let (hour, wind_speed, wind_gust, wind_direction) = item
			.next_tuple()
			.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"));
		data.push(HourlyWindData::from_args(
			hour,
			wind_speed,
			wind_gust,
			wind_direction,
		));
	}
	data
}

/// The highest cm/s the grid will display.
fn calculate_max_grid_wind_speed(data: &[HourlyWindData]) -> u16 {
	let highest_speed = data.iter().fold(u16::MIN, |acc, item| {
		acc.max(item.wind_gust).max(item.wind_speed)
	});
	let round_up = match highest_speed.rem_euclid(500) {
		0 => 0,
		n => 500 - n,
	};
	highest_speed + round_up
}

/// Draws the wind speed lines onto the canvas.
fn draw_wind_speed_lines(
	canvas: &mut RgbImage,
	data: &[HourlyWindData],
	grid_max_wind_speed: u16,
	gust: bool,
) {
	let gradient = if gust {
		MultiPointGradient::new(vec![
			GradientPoint::from_rgb(SPACE_BELOW, [70, 119, 67]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 7, [118, 118, 62]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 14, [122, 67, 62]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 21, [103, 78, 122]),
		])
	} else {
		MultiPointGradient::new(vec![
			GradientPoint::from_rgb(SPACE_BELOW, [0, 255, 33]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 7, [255, 255, 33]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 14, [255, 0, 33]),
			GradientPoint::from_rgb(SPACE_BELOW + PIXELS_PER_M_PER_S * 21, [188, 66, 255]),
		])
	};
	for (index, (start, end)) in data.iter().tuple_windows().enumerate() {
		let (start_speed, end_speed) = if gust {
			(start.wind_gust, end.wind_gust)
		} else {
			(start.wind_speed, end.wind_speed)
		};
		let start = Point {
			x: index as u32 * PIXELS_PER_HOUR + SPACE_LEFT,
			y: start_speed.abs_diff(grid_max_wind_speed) as u32 * PIXELS_PER_M_PER_S / 100
				+ SPACE_ABOVE,
		};
		let end = Point {
			x: (index + 1) as u32 * PIXELS_PER_HOUR + SPACE_LEFT,
			y: end_speed.abs_diff(grid_max_wind_speed) as u32 * PIXELS_PER_M_PER_S / 100
				+ SPACE_ABOVE,
		};
		draw_line_segment_with_gradient(canvas, start, end, &gradient);
	}
}
