use image::{Rgb, RgbImage};

use crate::{colours, common_types::Point};

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
