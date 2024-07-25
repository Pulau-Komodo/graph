use image::{codecs::png::PngEncoder, imageops, ColorType, ImageEncoder, RgbImage};

use crate::common_types::Range;

/// The highest value the chart will include.
pub fn next_multiple(highest: i32, interval: i32) -> i32 {
	let interval = interval * 100;
	let round_up = match highest.rem_euclid(interval) {
		0 => 0,
		n => interval - n,
	};
	highest + round_up
}

/// Get the lowest and highest values that the chart will include.
pub fn previous_and_next_multiple(range: Range<i32>, interval: i32) -> Range<i32> {
	let interval = interval * 100;
	let round_up = match range.end().rem_euclid(interval) {
		0 => 0,
		n => interval - n,
	};
	Range::new(
		range.start() - range.start().rem_euclid(interval),
		range.end() + round_up,
	)
}

pub fn make_png(canvas: RgbImage) -> Vec<u8> {
	let (width, height) = (canvas.width(), canvas.height());
	let mut buffer = Vec::new();
	let encoder = PngEncoder::new(&mut buffer);
	encoder
		.write_image(&canvas, width, height, ColorType::Rgb8.into())
		.unwrap();
	buffer
}

pub fn composite(images: &[RgbImage]) -> RgbImage {
	let max_width = images.iter().map(|image| image.width()).max().unwrap();
	let total_height = images.iter().map(|image| image.height()).sum::<u32>();
	let mut canvas = RgbImage::new(max_width, total_height);
	let mut last_height = 0_u32;
	for image in images {
		imageops::replace(&mut canvas, image, 0, last_height as i64);
		last_height += image.height();
	}
	canvas
}
