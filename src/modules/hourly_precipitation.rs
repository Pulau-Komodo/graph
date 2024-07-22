use ab_glyph::FontRef;
use image::RgbImage;

use crate::{
	colours,
	common_types::Range,
	drawing::{MarkIntervals, Padding, Spacing},
	from_args::{data_from_args, FromArgs},
	generic_graph::{AxisGridLabels, Chart, SolidBars},
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
const FONT_SCALE: ab_glyph::PxScale = ab_glyph::PxScale { x: 14.0, y: 14.0 };

pub fn parse_and_create(font: &FontRef<'static>, args: Vec<String>) -> RgbImage {
	let data = data_from_args(args);
	create(font, data)
}

pub fn create(font: &FontRef<'static>, data: Vec<HourlyPrecipitation>) -> RgbImage {
	let max_chart_precipitation = next_multiple(
		data.iter()
			.flat_map(|hour| [hour.rain as i32, hour.snow as i32])
			.max()
			.unwrap_or(0),
		1,
	) as u32;

	let mut chart = Chart::new(data.len(), max_chart_precipitation, SPACING, PADDING);

	chart.draw(AxisGridLabels {
		vertical_intervals: MarkIntervals::new(1, 1),
		horizontal_intervals: MarkIntervals::new(1, 2),
		vertical_label_range: Range::new(0, max_chart_precipitation as i32),
		horizontal_labels: data.iter().map(|hour| hour.hour),
		font: font.clone(),
		font_scale: FONT_SCALE,
	});
	chart.draw(SolidBars {
		colour: colours::RAIN,
		data: data.iter().map(|hour| hour.rain as i32),
	});
	chart.draw(SolidBars {
		colour: colours::SNOW,
		data: data.iter().map(|hour| hour.snow as i32),
	});

	chart.into_canvas()
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
