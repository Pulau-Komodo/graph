use graph::modules::{daily_temp, hourly_uvi};

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageFormat};

fn main() {
	let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
	let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to read font");
	let mut args = std::env::args();
	let mode = args.nth(1).expect("No arguments used");
	let args: Vec<_> = args.collect();
	let (canvas, to_file) = match mode.as_str() {
		"daily_temp" => (daily_temp::create(font, args), false),
		"hourly_uvi" => (hourly_uvi::create(font, args), false),
		"daily_temp_f" => (daily_temp::create(font, args), true),
		"hourly_uvi_f" => (hourly_uvi::create(font, args), true),
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
