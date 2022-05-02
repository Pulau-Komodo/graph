use graph::modules::daily_temp;

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageFormat};

fn main() {
	let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
	let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to read font");
	let mut args = std::env::args();
	let mode = args.nth(1).expect("No arguments used");
	let args: Vec<_> = args.collect();
	let canvas = match mode.as_str() {
		"daily_temp" => daily_temp::create(font, args),
		x => panic!("Unexpected first argument {x}"),
	};

	const TO_FILE: bool = true;
	if TO_FILE {
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
