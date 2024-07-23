use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, RgbImage};

pub mod daily_temp;
pub mod hourly_composite;
pub mod hourly_pop;
pub mod hourly_precipitation;
pub mod hourly_temp;
pub mod hourly_uvi;
pub mod hourly_wind;
pub mod minutely_precipitation;

pub fn make_png(canvas: RgbImage) -> Vec<u8> {
	let (width, height) = (canvas.width(), canvas.height());
	let mut buffer = Vec::new();
	let encoder = PngEncoder::new(&mut buffer);
	encoder
		.write_image(&canvas, width, height, ColorType::Rgb8.into())
		.unwrap();
	buffer
}
