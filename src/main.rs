use graph::modules::{
	daily_temp, hourly_composite, hourly_pop, hourly_precipitation, hourly_temp, hourly_uvi,
	hourly_wind, minutely_precipitation,
};

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageFormat};

fn main() {
	let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
	let font = ab_glyph::FontRef::try_from_slice(font_data).expect("Failed to read font");
	let mut args = std::env::args();
	let mut mode = args.nth(1).expect("No arguments used");
	let to_file = mode.as_str() == "file";
	if to_file {
		mode = args.next().expect("No arguments beyond \"file\"");
	}
	let args: Vec<_> = args.collect();
	let canvas = match mode.as_str() {
		"daily_temp" => daily_temp::parse_and_create(&font, args),
		"hourly_pop" => hourly_pop::parse_and_create(&font, args),
		"hourly_precipitation" => hourly_precipitation::parse_and_create(&font, args),
		"hourly_temp" => hourly_temp::parse_and_create(&font, args),
		"hourly_uvi" => hourly_uvi::parse_and_create(&font, args),
		"hourly_wind" => hourly_wind::parse_and_create(&font, args),
		"hourly_composite" => hourly_composite::parse_and_create(&font, args),
		"minutely_precipitation" => minutely_precipitation::parse_and_create(&font, args),
		x => panic!("Unexpected first argument {x}"),
	};

	if to_file {
		canvas
			.save_with_format("./image.png", ImageFormat::Png)
			.expect("Failed to save image");
	} else {
		let (width, height) = (canvas.width(), canvas.height());
		let encoder = PngEncoder::new(std::io::stdout());
		encoder
			.write_image(&canvas, width, height, ColorType::Rgb8.into())
			.expect("Failed to write image to stdout");
	}
}
