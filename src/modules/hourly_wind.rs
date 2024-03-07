use image::{Rgb, RgbImage};
use itertools::Itertools;
use rusttype::Font;

use crate::{
	colours,
	common_types::{GradientPoint, MultiPointGradient, Point, Range},
	drawing::{
		draw_graph_bars_with_gradient, draw_line_segment, draw_outer_lines, fill_canvas,
		horizontal_lines_and_labels, vertical_lines_and_labels, MarkIntervals, Padding, Spacing,
	},
	util::next_multiple,
};

const PADDING: Padding = Padding {
	above: 7,
	below: 19 + 12 + 3,
	left: 21,
	right: 3,
};
const SPACING: Spacing = Spacing {
	horizontal: 8,
	vertical: 5,
};
const DIRECTION_GRAPH_HEIGHT: u32 = 13;
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &Font, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &Font, data: Vec<HourlyWind>) -> RgbImage {
	let max_chart_speed = next_multiple(
		data.iter()
			.flat_map(|hour| [hour.wind_speed, hour.wind_gust])
			.max()
			.unwrap_or(0) as i32,
		5,
	);
	let width = data.len() as u32 * SPACING.horizontal + PADDING.horizontal();
	let height = max_chart_speed as u32 * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, colours::BACKGROUND);
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|data| data.hour),
		MarkIntervals::new(1, 2),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		Range::new(0, max_chart_speed),
		MarkIntervals::new(5, 5),
		font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	let gradient = MultiPointGradient::new(vec![
		GradientPoint::from_rgb(PADDING.below, [70, 119, 67]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 7, [118, 118, 62]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 14, [122, 67, 62]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 21, [103, 78, 122]),
	]);
	draw_graph_bars_with_gradient(
		&mut canvas,
		data.iter().map(|hour| hour.wind_gust as i32),
		gradient,
		PADDING,
		SPACING,
	);
	let gradient = MultiPointGradient::new(vec![
		GradientPoint::from_rgb(PADDING.below, [0, 255, 33]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 7, [255, 255, 33]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 14, [255, 0, 33]),
		GradientPoint::from_rgb(PADDING.below + SPACING.vertical * 21, [188, 66, 255]),
	]);
	draw_graph_bars_with_gradient(
		&mut canvas,
		data.iter().map(|hour| hour.wind_speed as i32),
		gradient,
		PADDING,
		SPACING,
	);
	let x = PADDING.left;
	let x2 = width - PADDING.right - 1;
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

pub struct HourlyWind {
	/// Hour of the day
	hour: u8,
	/// Wind speed in cm/s
	wind_speed: u16,
	/// Wind gust speed in cm/s
	wind_gust: u16,
	/// Wind direction in degrees, where 0 is north and 90 is east
	wind_direction: u16,
}

impl HourlyWind {
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

fn data_from_args(args: Vec<String>) -> Vec<HourlyWind> {
	const CHUNK_SIZE: usize = 4;
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let (hour, wind_speed, wind_gust, wind_direction) = item
			.next_tuple()
			.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"));
		data.push(HourlyWind::from_args(
			hour,
			wind_speed,
			wind_gust,
			wind_direction,
		));
	}
	data
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
			AngleInterpolation::new(start, end, SPACING.horizontal as u16).enumerate()
		{
			for (y, colour) in WindDirectionPixelColumn::new(direction).enumerate() {
				if let Some(colour) = colour {
					canvas.put_pixel(
						hour_count as u32 * SPACING.horizontal + PADDING.left + x as u32,
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
