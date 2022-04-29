use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageFormat, Rgb, RgbImage};
use itertools::Itertools;

const SPACE_ABOVE: u32 = 7;
const SPACE_BELOW: u32 = 19;
const SPACE_LEFT: u32 = 21;
const SPACE_RIGHT: u32 = 9;
const PIXELS_PER_CELSIUS: u32 = 3;
const PIXELS_PER_DAY: u32 = 25;

fn main() {
	let data = day_data_from_args();
	let (temp_range_min, temp_range_max) = {
		let (all_temps_min, all_temps_max) = data.iter().fold(
			(i32::MAX, i32::MIN),
			|(min, max),
			 DayData {
			     temp_min, temp_max, ..
			 }| { (min.min(*temp_min), max.max(*temp_max)) },
		);
		let round_up = match all_temps_max.rem_euclid(400) {
			0 => 0,
			n => 400 - n,
		};
		(
			all_temps_min - all_temps_min.rem_euclid(400),
			all_temps_max + round_up,
		)
	};
	let temp_range = temp_range_max.abs_diff(temp_range_min);
	let width = PIXELS_PER_DAY * (data.len() - 1) as u32 + SPACE_LEFT + SPACE_RIGHT;
	let height = temp_range * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE + SPACE_BELOW;
	let mut canvas = RgbImage::new(width, height);
	imageproc::drawing::draw_filled_rect_mut(
		&mut canvas,
		imageproc::rect::Rect::at(0, 0).of_size(width, height),
		Rgb::<u8>([0, 0, 0]),
	);
	let colour_min = Rgb::<u8>([0, 148, 255]);
	let colour_max = Rgb::<u8>([255, 0, 0]);
	let colour_main_lines = Rgb::<u8>([127, 127, 127]);
	let colour_grid_lines = Rgb::<u8>([63, 63, 63]);
	let colour_text = Rgb::<u8>([127, 127, 127]);
	let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
	let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to read font");
	let font_scale = rusttype::Scale { x: 14.0, y: 14.0 };
	/*for (
		index,
		DayData {
			temp_min, temp_max, ..
		},
	) in data.iter().enumerate()
	{
		let point_temp_min = Point {
			x: index as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: temp_min.abs_diff(all_temps_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		place_dot(&mut canvas, point_temp_min, colour_min);
		let point_temp_max = Point {
			x: index as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: temp_max.abs_diff(all_temps_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		place_dot(&mut canvas, point_temp_max, colour_max);
	}*/
	let x = SPACE_LEFT - 1;
	draw_line_segment(
		&mut canvas,
		Point { x, y: SPACE_ABOVE },
		Point {
			x,
			y: height - SPACE_BELOW,
		},
		colour_main_lines,
	);
	let y = height - SPACE_BELOW + 1;
	draw_line_segment(
		&mut canvas,
		Point {
			x: SPACE_LEFT - 1,
			y,
		},
		Point {
			x: width - SPACE_RIGHT,
			y,
		},
		colour_main_lines,
	);
	for (index, day) in data.iter().enumerate() {
		let x = SPACE_LEFT + index as u32 * PIXELS_PER_DAY;
		draw_line_segment(
			&mut canvas,
			Point { x, y: SPACE_ABOVE },
			Point {
				x,
				y: height - SPACE_BELOW,
			},
			colour_grid_lines,
		);
		let text = &format!("{}", day.day);
		let (text_width, _text_height) = imageproc::drawing::text_size(font_scale, &font, text);
		imageproc::drawing::draw_text_mut(
			&mut canvas,
			colour_text,
			x as i32 - text_width / 2,
			(height - SPACE_BELOW + 5) as i32,
			font_scale,
			&font,
			text,
		);
	}
	for temp in (temp_range_min / 100..=temp_range_max / 100).step_by(2) {
		let y = SPACE_ABOVE + (temp_range_max / 100).abs_diff(temp) * PIXELS_PER_CELSIUS;
		let line_colour = if temp == 0 {
			colour_main_lines
		} else {
			colour_grid_lines
		};
		draw_line_segment(
			&mut canvas,
			Point { x: SPACE_LEFT, y },
			Point {
				x: width - SPACE_RIGHT,
				y,
			},
			line_colour,
		);
		if temp % 4 == 0 {
			let text = &format!("{}", temp);
			let (text_width, text_height) = imageproc::drawing::text_size(font_scale, &font, text);
			imageproc::drawing::draw_text_mut(
				&mut canvas,
				colour_text,
				SPACE_LEFT as i32 - text_width - 3,
				y as i32 - text_height / 2,
				font_scale,
				&font,
				text,
			);
		}
	}
	for (index, (start, end)) in data.iter().tuple_windows().enumerate() {
		let start = Point {
			x: index as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: start.temp_min.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		let end = Point {
			x: (index + 1) as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: end.temp_min.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		draw_line_segment(&mut canvas, start, end, colour_min);
	}
	for (index, (start, end)) in data.iter().tuple_windows().enumerate() {
		let start = Point {
			x: index as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: start.temp_max.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		let end = Point {
			x: (index + 1) as u32 * PIXELS_PER_DAY + SPACE_LEFT,
			y: end.temp_max.abs_diff(temp_range_max) * PIXELS_PER_CELSIUS / 100 + SPACE_ABOVE,
		};
		draw_line_segment(&mut canvas, start, end, colour_max);
	}

	const TO_FILE: bool = false;
	if TO_FILE {
		canvas
			.save_with_format("./image.png", ImageFormat::Png)
			.expect("Failed to save image");
	} else {
		let encoder = PngEncoder::new(std::io::stdout());
		encoder
			.write_image(&canvas, width, height, ColorType::Rgb8)
			.expect("Failed to write image to stdout");
	}
}

#[derive(Debug, Clone, Copy)]
struct DayData {
	/// Day of the month
	day: u8,
	/// Minimum temperature in centidegrees Celsius
	temp_min: i32,
	/// Maximum temperature in centidegrees Celsius
	temp_max: i32,
}

impl DayData {
	fn from_args(day: String, temp_min: String, temp_max: String) -> Self {
		let day = day.parse().expect("Could not parse a day argument");
		let temp_min = temp_min
			.parse()
			.expect("Could not parse a minimum temperature argument");
		let temp_max = temp_max
			.parse()
			.expect("Could not parse a maximum temperature argument");
		Self {
			day,
			temp_min,
			temp_max,
		}
	}
}

fn day_data_from_args() -> Vec<DayData> {
	let mut pairs = Vec::with_capacity(std::env::args().skip(1).count() / 2);
	for mut item in std::env::args().skip(1).chunks(3).into_iter() {
		let (day, temp_min, temp_max) = item
			.next_tuple()
			.expect("Arguments could not be grouped into threes");
		let data = DayData::from_args(day, temp_min, temp_max);
		pairs.push(data);
	}
	pairs
}

struct Point {
	x: u32,
	y: u32,
}

fn place_dot(canvas: &mut RgbImage, point: Point, colour: Rgb<u8>) {
	canvas.put_pixel(point.x, point.y, colour);
	canvas.put_pixel(point.x - 1, point.y, colour);
	canvas.put_pixel(point.x + 1, point.y, colour);
	canvas.put_pixel(point.x, point.y - 1, colour);
	canvas.put_pixel(point.x, point.y + 1, colour);
}

fn draw_line_segment(canvas: &mut RgbImage, start: Point, end: Point, colour: Rgb<u8>) {
	for point in BresenhamLineIter::new(start, end) {
		canvas.put_pixel(point.x, point.y, colour);
	}
}

/// Iterates over the coordinates in a line segment using
/// [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm).
/// Stolen/adapted from imageproc crate
struct BresenhamLineIter {
	dx: u32,
	dy: u32,
	x: u32,
	y: u32,
	error: f32,
	end_x: u32,
	is_steep: bool,
	is_ascending: bool,
}

impl BresenhamLineIter {
	/// Creates a [`BresenhamLineIter`](struct.BresenhamLineIter.html) which will iterate over the integer coordinates
	/// between `start` and `end`.
	fn new(start: Point, end: Point) -> BresenhamLineIter {
		let Point {
			x: mut x0,
			y: mut y0,
		} = start;
		let Point {
			x: mut x1,
			y: mut y1,
		} = end;

		let is_steep = y1.abs_diff(y0) > x1.abs_diff(x0);
		if is_steep {
			std::mem::swap(&mut x0, &mut y0);
			std::mem::swap(&mut x1, &mut y1);
		}

		if x0 > x1 {
			std::mem::swap(&mut x0, &mut x1);
			std::mem::swap(&mut y0, &mut y1);
		}

		let dx = x1 - x0;

		BresenhamLineIter {
			dx,
			dy: y1.abs_diff(y0),
			x: x0,
			y: y0,
			error: dx as f32 / 2f32,
			end_x: x1,
			is_steep,
			is_ascending: y0 < y1,
		}
	}
}

impl Iterator for BresenhamLineIter {
	type Item = Point;

	fn next(&mut self) -> Option<Point> {
		if self.x > self.end_x {
			None
		} else {
			let ret = if self.is_steep {
				Point {
					x: self.y,
					y: self.x,
				}
			} else {
				Point {
					x: self.x,
					y: self.y,
				}
			};

			self.x += 1;
			self.error -= self.dy as f32;
			if self.error < 0f32 {
				if self.is_ascending {
					self.y += 1;
				} else {
					self.y -= 1;
				}
				self.error += self.dx as f32;
			}

			Some(ret)
		}
	}
}
