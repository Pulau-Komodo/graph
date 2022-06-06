use graph::modules::{
	daily_temp, hourly_pop, hourly_precipitation, hourly_temp, hourly_uvi, hourly_wind,
};

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageFormat};

fn main() {
	let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
	let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to read font");
	let mut args = std::env::args();
	let mut mode = args.nth(1).expect("No arguments used");
	let to_file = mode.as_str() == "file";
	if to_file {
		mode = args.next().expect("No arguments beyond \"file\"");
	}
	let args: Vec<_> = args.collect();
	let canvas = match mode.as_str() {
		"daily_temp" => daily_temp::create(font, args),
		"hourly_pop" => hourly_pop::create(font, args),
		"hourly_precipitation" => hourly_precipitation::create(font, args),
		"hourly_temp" => hourly_temp::create(font, args),
		"hourly_uvi" => hourly_uvi::create(font, args),
		"hourly_wind" => hourly_wind::create(font, args),
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
			.write_image(&canvas, width, height, ColorType::Rgb8)
			.expect("Failed to write image to stdout");
	}
}
