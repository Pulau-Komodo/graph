use ab_glyph::FontRef;
use image::{imageops, RgbImage};

use crate::modules::{hourly_pop, hourly_precipitation, hourly_temp, hourly_uvi, hourly_wind};

pub fn parse_and_create(font: &FontRef, args: Vec<String>) -> RgbImage {
	let mut component_args = args.into_iter();
	let temp_args = component_args.next().expect("No temperature arguments");
	let temp_args = temp_args.split(' ').map(String::from).collect::<Vec<_>>();
	let temp_graph = hourly_temp::parse_and_create(font, temp_args);
	let pop_args = component_args
		.next()
		.expect("No probability of precipitation arguments");
	let pop_args = pop_args.split(' ').map(String::from).collect::<Vec<_>>();
	let pop_graph = hourly_pop::parse_and_create(font, pop_args);
	let precipitation_args = component_args.next().expect("No precipitation arguments");
	let precipitation_args = precipitation_args
		.split(' ')
		.map(String::from)
		.collect::<Vec<_>>();
	let precipitation_graph = hourly_precipitation::parse_and_create(font, precipitation_args);
	let wind_args = component_args.next().expect("No wind arguments");
	let wind_args = wind_args.split(' ').map(String::from).collect::<Vec<_>>();
	let wind_graph = hourly_wind::parse_and_create(font, wind_args);
	let uvi_args = component_args.next().expect("No uvi arguments");
	let uvi_args = uvi_args.split(' ').map(String::from).collect::<Vec<_>>();
	let uvi_graph = hourly_uvi::parse_and_create(font, uvi_args);
	composite([
		temp_graph,
		pop_graph,
		precipitation_graph,
		wind_graph,
		uvi_graph,
	])
}

fn composite(images: [RgbImage; 5]) -> RgbImage {
	let max_width = images.iter().map(|image| image.width()).max().unwrap();
	let total_height = images.iter().map(|image| image.height()).sum::<u32>();
	let mut canvas = RgbImage::new(max_width, total_height);
	let mut last_height = 0_u32;
	for image in images {
		imageops::replace(&mut canvas, &image, 0, last_height as i64);
		last_height += image.height();
	}
	canvas
}
