use image::{Rgb, RgbImage};
use rusttype::Font;

use crate::{
	common_types::Range,
	drawing::{
		draw_graph_lines, draw_outer_lines, fill_canvas, horizontal_lines_and_labels,
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
	vertical: 3,
};
const FONT_SCALE: rusttype::Scale = rusttype::Scale { x: 14.0, y: 14.0 };

pub fn create(font: Font, args: Vec<String>) -> RgbImage {
	let data: Vec<HourlyTempData> = data_from_args(args);
	let temp_range = calculate_grid_range(&data);
	let width = SPACING.horizontal * (data.len() - 1) as u32 + PADDING.horizontal();
	let height = temp_range.len() as u32 * SPACING.vertical / 100 + PADDING.vertical();
	let mut canvas = RgbImage::new(width, height);
	fill_canvas(&mut canvas, Rgb::<u8>([0, 0, 0]));
	draw_outer_lines(&mut canvas, PADDING);
	vertical_lines_and_labels(
		&mut canvas,
		data.iter().map(|hour| hour.hour),
		MarkIntervals::new(1, 2),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.horizontal,
	);
	horizontal_lines_and_labels(
		&mut canvas,
		temp_range,
		MarkIntervals::new(2, 4),
		&font,
		FONT_SCALE,
		PADDING,
		SPACING.vertical,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|hour| hour.feels_like),
		Rgb([0, 255, 33]),
		temp_range.end(),
		PADDING,
		SPACING,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|hour| hour.wet_bulb),
		Rgb([0, 148, 255]),
		temp_range.end(),
		PADDING,
		SPACING,
	);
	draw_graph_lines(
		&mut canvas,
		data.iter().map(|hour| hour.temp),
		Rgb([255, 0, 0]),
		temp_range.end(),
		PADDING,
		SPACING,
	);
	/*for (index, wet_bulb) in data
		.iter()
		.enumerate()
		.filter(|(_index, hour)| hour.wet_bulb_is_accurate)
		.map(|(index, hour)| (index, hour.wet_bulb))
	{

	}*/
	canvas
}

#[derive(Debug, Clone, Copy)]
struct HourlyTempData {
	/// Hour of the day
	hour: u8,
	/// Dry-bulb temperature in centidegrees Celsius
	temp: i32,
	/// Feels-like temperature in centidegrees Celsius
	feels_like: i32,
	/// Wet-bulb temperature in centidegrees Celsius
	wet_bulb: i32,
	/// Whether the wet-bulb temperature is accurate (if not, the input was outside the range the calculation was valid for)
	_wet_bulb_is_accurate: bool,
}

impl FromArgs<4> for HourlyTempData {
	fn from_args([hour, temp, feels_like, humidity]: [String; 4]) -> Self {
		let hour = hour.parse().expect("Could not parse an hour argument");
		let temp = temp
			.parse()
			.expect("Could not parse a dry-bulb temperature argument");
		let feels_like = feels_like
			.parse()
			.expect("Could not parse a feels-like temperature argument");
		let humidity: u8 = humidity
			.parse()
			.expect("Could not parse a humidity argument");
		let wet_bulb = if humidity == 100 {
			temp
		} else {
			(wet_bulb_temp(temp as f32 / 100.0, humidity as f32) * 100.0).round() as i32
		};
		let wet_bulb_is_accurate = (-2000..5000).contains(&temp) && humidity >= 5;
		Self {
			hour,
			temp,
			feels_like,
			wet_bulb,
			_wet_bulb_is_accurate: wet_bulb_is_accurate,
		}
	}
}

/// Calculates wet bulb temperature in °C given dry bulb temperature in °C and relative humidity * 100 (0-100).
///
/// Supposedly this is only accurate for temperatures between -20 °C and 50 °C, and relative humidities between .05 and .99 (5 and 99).
fn wet_bulb_temp(temp: f32, humidity: f32) -> f32 {
	temp * (0.15197 * (humidity + 8.313659).sqrt()).atan() + (temp + humidity).atan()
		- (humidity - 1.676331).atan()
		+ 0.00391838 * humidity.powf(1.5) * (0.023101 * humidity).atan()
		- 4.686035
}

/// Get the lowest and highest temperatures that the grid will show. These are the closest multiples of 4 degrees Celsius that include all data.
fn calculate_grid_range(data: &[HourlyTempData]) -> Range<i32> {
	let (all_temps_min, all_temps_max) = data
		.iter()
		.flat_map(
			|HourlyTempData {
			     temp,
			     feels_like,
			     wet_bulb,
			     ..
			 }| [temp, feels_like, wet_bulb].into_iter(),
		)
		.fold((i32::MAX, i32::MIN), |(min, max), &temp| {
			(min.min(temp), max.max(temp))
		});
	let round_up = match all_temps_max.rem_euclid(400) {
		0 => 0,
		n => 400 - n,
	};
	Range::new(
		all_temps_min - all_temps_min.rem_euclid(400),
		all_temps_max + round_up,
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wet_bulb() {
		let tests = [
			(-10, 20, -11),
			(-10, 35, -12),
			(-10, 50, -12),
			(0, 20, -5),
			(0, 80, -2),
			(30, 20, 16),
			(30, 50, 22),
		];
		for (temp, humidity, wet_bulb) in tests {
			let temp = (temp * 100) as f32 / 100.0;
			let result = (wet_bulb_temp(temp, humidity as f32)).round() as i32;
			assert_eq!(result, wet_bulb);
		}
	}
}
