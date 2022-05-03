use image::{Rgb, RgbImage};
use rusttype::Scale;

use crate::{
	colours,
	common_types::{MultiPointGradient, Point, Range},
};

pub fn draw_line_segment(
	canvas: &mut RgbImage,
	start: Point<u32>,
	end: Point<u32>,
	colour: Rgb<u8>,
) {
	for point in BresenhamLineIter::new(start, end) {
		canvas.put_pixel(point.x, point.y, colour);
	}
}

/// Gradient based on height
pub fn draw_line_segment_with_gradient(
	canvas: &mut RgbImage,
	start: Point<u32>,
	end: Point<u32>,
	gradient: &MultiPointGradient,
) {
	for point in BresenhamLineIter::new(start, end) {
		let gradient_point = canvas.height() - point.y;
		let colour = Rgb(gradient.get_colour(gradient_point));
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
	fn new(start: Point<u32>, end: Point<u32>) -> BresenhamLineIter {
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
	type Item = Point<u32>;

	fn next(&mut self) -> Option<Point<u32>> {
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

pub struct Padding {
	pub above: u32,
	pub below: u32,
	pub left: u32,
	pub right: u32,
}

pub fn draw_outer_lines(canvas: &mut RgbImage, padding: Padding) {
	let height = canvas.height();
	let x = padding.left - 1;
	draw_line_segment(
		canvas,
		Point {
			x,
			y: padding.above,
		},
		Point {
			x,
			y: height - padding.below,
		},
		colours::MAIN_LINES,
	);
	let y = height - padding.below + 1;
	let width = canvas.width();
	draw_line_segment(
		canvas,
		Point {
			x: padding.left - 1,
			y,
		},
		Point {
			x: width - padding.right,
			y,
		},
		colours::MAIN_LINES,
	);
}

pub fn fill_canvas(canvas: &mut RgbImage, colour: Rgb<u8>) {
	let width = canvas.width();
	let height = canvas.height();
	imageproc::drawing::draw_filled_rect_mut(
		canvas,
		imageproc::rect::Rect::at(0, 0).of_size(width, height),
		colour,
	);
}

pub fn horizontal_lines_and_labels(
	canvas: &mut RgbImage,
	data_range: Range<i32>,
	line_interval: usize,
	label_interval: usize,
	font: &rusttype::Font,
	font_scale: Scale,
	padding: Padding,
	spacing: u32,
) {
	let width = canvas.width();
	let max_value = data_range.end() / 100;
	for value in (data_range.start()..=data_range.end())
		.step_by(line_interval * 100)
		.map(|n| n / 100)
	{
		let y = padding.above + max_value.abs_diff(value) * spacing;
		let line_colour = if value == 0 {
			colours::MAIN_LINES
		} else {
			colours::GRID_LINES
		};
		draw_line_segment(
			canvas,
			Point { x: padding.left, y },
			Point {
				x: width - padding.right,
				y,
			},
			line_colour,
		);
		if value % label_interval as i32 == 0 {
			let text = &format!("{}", value);
			let (text_width, text_height) = imageproc::drawing::text_size(font_scale, font, text);
			imageproc::drawing::draw_text_mut(
				canvas,
				colours::TEXT,
				padding.left as i32 - text_width - 3,
				y as i32 - text_height / 2,
				font_scale,
				font,
				text,
			);
		}
	}
}

pub fn vertical_lines_and_labels(
	canvas: &mut RgbImage,
	data: impl Iterator<Item = u8>,
	line_interval: usize,
	label_interval: usize,
	font: &rusttype::Font,
	font_scale: Scale,
	padding: Padding,
	spacing: u32,
) {
	let height = canvas.height();
	for (index, item) in data.enumerate().step_by(line_interval) {
		let x = padding.left + index as u32 * spacing;
		draw_line_segment(
			canvas,
			Point {
				x,
				y: padding.above,
			},
			Point {
				x,
				y: height - padding.below,
			},
			colours::GRID_LINES,
		);
		if index % label_interval == 0 {
			let text = &format!("{}", item);
			let (text_width, _text_height) = imageproc::drawing::text_size(font_scale, font, text);
			imageproc::drawing::draw_text_mut(
				canvas,
				colours::TEXT,
				x as i32 - text_width / 2,
				(height - padding.below + 5) as i32,
				font_scale,
				font,
				text,
			);
		}
	}
}
