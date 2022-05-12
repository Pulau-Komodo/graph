use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	common_types::{GradientPoint, MultiPointGradient, Point, Range},
	drawing::{
		draw_line_segment, draw_line_segment_with_gradient, draw_outer_lines, fill_canvas,
		horizontal_lines_and_labels, vertical_lines_and_labels, Padding,
	},
};

const SPACE_ABOVE: u32 = 7;
const SPACE_BELOW: u32 = 19 + 12 + 3;
const SPACE_LEFT: u32 = 21;
const SPACE_RIGHT: u32 = 3;
const PIXELS_PER_M_PER_S: u32 = 5;
const PIXELS_PER_HOUR: u32 = 8;
const DIRECTION_GRAPH_HEIGHT: u32 = 13;
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
	let x = SPACE_LEFT;
	let x2 = width - SPACE_RIGHT - 1;
	let y = height - DIRECTION_GRAPH_HEIGHT / 2 - 3;
	for n in (0..3).step_by(2) {
		let y = y - 1 + n;
		draw_line_segment(
			&mut canvas,
			Point { x, y },
			Point { x: x2, y },
			Rgb([255, 255, 255]),
		);
	}
	draw_wind_directions(&mut canvas, data.iter().map(|hour| hour.wind_direction));
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

struct AngleInterpolation {
	start: u16,
	distance: u16,
	/// Might max out at 364? 242? due to overflowing
	steps: u16,
	is_clockwise: bool,
	current: u16,
}

impl AngleInterpolation {
	fn new(start: u16, end: u16, steps: u16) -> Self {
		if start > 360 {
			panic!("Start is too high ({start} > 360)");
		}
		if end > 360 {
			panic!("End is too high ({end} > 360)");
		}
		let (start, end) = (start % 360, end % 360);
		let abs_diff = start.abs_diff(end);
		//let is_clockwise = (abs_diff < 180) ^ (start > end);
		let is_clockwise = (start + 360 - end) % 360 > 180;
		let distance = if abs_diff > 180 {
			360 - abs_diff
		} else {
			abs_diff
		};
		Self {
			start,
			distance,
			steps,
			is_clockwise,
			current: 1,
		}
	}
}

impl Iterator for AngleInterpolation {
	type Item = u16;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current > self.steps {
			return None;
		}
		let value = (self.current * self.distance + self.steps / 2) / self.steps;
		self.current += 1;
		if self.is_clockwise {
			Some((self.start + value) % 360)
		} else {
			Some((self.start + 360 - value) % 360)
		}
	}
}

struct WindDirectionPixelColumn {
	direction_offset: usize,
	current: usize,
}

impl WindDirectionPixelColumn {
	fn new(direction: u16) -> Self {
		let direction_offset = ((direction * 20 + 180) / 360) as usize;
		Self {
			direction_offset,
			current: 0,
		}
	}
}

impl Iterator for WindDirectionPixelColumn {
	type Item = Option<[u8; 3]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= DIRECTION_GRAPH_HEIGHT as usize {
			return None;
		}
		const NORTH_LINE: [u8; 3] = [255, 0, 0];
		const EAST_LINE: [u8; 3] = [181, 150, 0];
		const SOUTH_LINE: [u8; 3] = [0, 148, 255];
		const WEST_LINE: [u8; 3] = [178, 0, 255];
		const COLOURS: [Option<[u8; 3]>; 20] = [
			None,
			None,
			Some(NORTH_LINE),
			None,
			None,
			None,
			None,
			Some(EAST_LINE),
			None,
			None,
			None,
			None,
			Some(SOUTH_LINE),
			None,
			None,
			None,
			None,
			Some(WEST_LINE),
			None,
			None,
		];
		let colour = COLOURS
			[(self.current + self.direction_offset + DIRECTION_GRAPH_HEIGHT as usize / 2 + 9) % 20];
		self.current += 1;
		Some(colour)
	}
}

fn draw_wind_directions(canvas: &mut RgbImage, directions: impl IntoIterator<Item = u16>) {
	let height = canvas.height();
	for (hour_count, (start, end)) in directions.into_iter().tuple_windows().enumerate() {
		for (x, direction) in
			AngleInterpolation::new(start, end, PIXELS_PER_HOUR as u16).enumerate()
		{
			for (y, colour) in WindDirectionPixelColumn::new(direction).enumerate() {
				if let Some(colour) = colour {
					canvas.put_pixel(
						hour_count as u32 * PIXELS_PER_HOUR + SPACE_LEFT + x as u32,
						height - 16 + y as u32,
						Rgb(colour),
					);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn interpolate_angle() {
		assert_eq!(
			vec![30, 60, 90],
			AngleInterpolation::new(0, 90, 3).collect::<Vec<_>>()
		);
		assert_eq!(
			vec![60, 30, 0],
			AngleInterpolation::new(90, 0, 3).collect::<Vec<_>>()
		);
		assert_eq!(
			vec![345, 15, 45],
			AngleInterpolation::new(315, 45, 3).collect::<Vec<_>>()
		);
		assert_eq!(
			vec![15, 345, 315],
			AngleInterpolation::new(45, 315, 3).collect::<Vec<_>>()
		);
	}
	#[test]
	fn interpolate_rounding() {
		assert_eq!(
			vec![33, 67, 100],
			AngleInterpolation::new(0, 100, 3).collect::<Vec<_>>()
		);
		assert_eq!(
			vec![67, 33, 0],
			AngleInterpolation::new(100, 0, 3).collect::<Vec<_>>()
		);
	}
	#[test]
	fn get_direction_colours() {
		for (colour_1, colour_2) in
			WindDirectionPixelColumn::new(0).zip(WindDirectionPixelColumn::new(180).skip(10))
		{
			assert_eq!(colour_1, colour_2);
		}
	}
}
