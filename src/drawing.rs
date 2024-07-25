use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};
use itertools::Itertools;

use crate::{
	colours,
	common_types::{MultiPointGradient, Point, Range},
};

pub(crate) fn draw_line_segment(
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
pub(crate) fn _draw_line_segment_with_gradient(
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

/// Gradient based on height
pub(crate) fn draw_bar_with_gradient(
	canvas: &mut RgbImage,
	rectangle: Rect,
	gradient: &MultiPointGradient,
) {
	for y in rectangle.top()..=rectangle.bottom() {
		let gradient_point = canvas.height() - y as u32;
		let colour = Rgb(gradient.get_colour(gradient_point));
		for x in rectangle.left()..=rectangle.right() {
			canvas.put_pixel(x as u32, y as u32, colour)
		}
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

#[derive(Debug, Clone, Copy)]
pub struct Padding {
	pub above: u32,
	pub below: u32,
	pub left: u32,
	pub right: u32,
}

impl Padding {
	/** This is just left + right. */
	pub(crate) const fn horizontal(&self) -> u32 {
		self.left + self.right
	}
	/** This is just above + below. */
	pub(crate) const fn vertical(&self) -> u32 {
		self.above + self.below
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Spacing {
	pub horizontal: u32,
	pub vertical: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct MarkIntervals {
	line: usize,
	label: usize,
}

impl MarkIntervals {
	/** Panics if label is not a multiple of line */
	pub(crate) const fn new(line: usize, label: usize) -> Self {
		if label % line != 0 {
			panic!("Labelling interval needs to be a multiple of line drawing interval.");
		}
		Self { line, label }
	}
	pub(crate) const fn line(&self) -> usize {
		self.line
	}
	pub(crate) const fn label(&self) -> usize {
		self.label
	}
}

pub(crate) fn draw_outer_lines(canvas: &mut RgbImage, padding: Padding) {
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

pub(crate) fn fill_canvas(canvas: &mut RgbImage, colour: Rgb<u8>) {
	let width = canvas.width();
	let height = canvas.height();
	imageproc::drawing::draw_filled_rect_mut(
		canvas,
		imageproc::rect::Rect::at(0, 0).of_size(width, height),
		colour,
	);
}

pub(crate) fn horizontal_lines_and_labels(
	canvas: &mut RgbImage,
	data_range: Range<i32>,
	intervals: MarkIntervals,
	font: &FontRef,
	font_scale: PxScale,
	padding: Padding,
	spacing: u32,
) {
	let width = canvas.width();
	let max_value = data_range.end() / 100;
	for value in (data_range.start()..=data_range.end())
		.step_by(intervals.line() * 100)
		.map(|n| n / 100)
	{
		let y = padding.above + max_value.abs_diff(value) * spacing;
		let line_colour = if value == 0 {
			colours::MAIN_LINES
		} else if value / 100 % intervals.label() as i32 == 0 {
			colours::BRIGHTER_GRID_LINES
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
		if value % intervals.label() as i32 == 0 {
			let text = &format!("{}", value);
			let (text_width, text_height) = imageproc::drawing::text_size(font_scale, &font, text);
			let text_width = text_width as i32;
			let text_height = text_height as i32;
			imageproc::drawing::draw_text_mut(
				canvas,
				colours::TEXT,
				padding.left as i32 - text_width - 3,
				y as i32 - text_height / 2,
				font_scale,
				&font,
				text,
			);
		}
	}
}

pub(crate) fn vertical_lines_and_labels(
	canvas: &mut RgbImage,
	data: impl Iterator<Item = u8>,
	intervals: MarkIntervals,
	font: &FontRef,
	font_scale: PxScale,
	padding: Padding,
	spacing: u32,
	center: bool,
) {
	let height = canvas.height();
	let mut count = 0;
	for (index, item) in data.enumerate() {
		count = index;
		if index % intervals.line() != 0 {
			continue;
		}
		let x = padding.left + index as u32 * spacing;
		let line_colour = if index % intervals.label() == 0 {
			colours::BRIGHTER_GRID_LINES
		} else {
			colours::GRID_LINES
		};
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
			line_colour,
		);
		if index % intervals.label() == 0 {
			let text = &format!("{}", item);
			let (text_width, _text_height) = imageproc::drawing::text_size(font_scale, &font, text);
			let text_width = text_width as i32;
			let x = if center {
				x as i32 - (text_width - spacing as i32) / 2
			} else {
				x as i32 - text_width / 2
			};
			imageproc::drawing::draw_text_mut(
				canvas,
				colours::TEXT,
				x,
				(height - padding.below + 5) as i32,
				font_scale,
				&font,
				text,
			);
		}
	}
	count += 1;
	if center && count % intervals.line() == 0 {
		let x = padding.left + count as u32 * spacing;
		if x >= canvas.width() {
			return;
		}
		let line_colour = if count % intervals.label() == 0 {
			colours::BRIGHTER_GRID_LINES
		} else {
			colours::GRID_LINES
		};
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
			line_colour,
		);
	}
}

/// Draws the line graph lines onto the canvas.
pub(crate) fn draw_graph_lines(
	canvas: &mut RgbImage,
	data: impl IntoIterator<Item = i32>,
	colour: Rgb<u8>,
	max: i32,
	padding: Padding,
	spacing: Spacing,
) {
	for (index, (start, end)) in data.into_iter().tuple_windows().enumerate() {
		let start = Point {
			x: index as u32 * spacing.horizontal + padding.left,
			y: start.abs_diff(max) * spacing.vertical / 100 + padding.above,
		};
		let end = Point {
			x: (index + 1) as u32 * spacing.horizontal + padding.left,
			y: end.abs_diff(max) * spacing.vertical / 100 + padding.above,
		};
		draw_line_segment(canvas, start, end, colour);
	}
}

/// Draws the line graph lines onto the canvas with a height-based gradient.
pub(crate) fn _draw_graph_lines_with_gradient(
	canvas: &mut RgbImage,
	data: impl IntoIterator<Item = i32>,
	gradient: MultiPointGradient,
	max: i32,
	padding: Padding,
	spacing: Spacing,
) {
	for (index, (start, end)) in data.into_iter().tuple_windows().enumerate() {
		let start = Point {
			x: index as u32 * spacing.horizontal + padding.left,
			y: start.abs_diff(max) * spacing.vertical / 100 + padding.above,
		};
		let end = Point {
			x: (index + 1) as u32 * spacing.horizontal + padding.left,
			y: end.abs_diff(max) * spacing.vertical / 100 + padding.above,
		};
		_draw_line_segment_with_gradient(canvas, start, end, &gradient);
	}
}

pub(crate) fn draw_graph_bars(
	canvas: &mut RgbImage,
	data: impl IntoIterator<Item = i32>,
	colour: Rgb<u8>,
	padding: Padding,
	spacing: Spacing,
) {
	let height = canvas.height();
	for (index, value) in data.into_iter().enumerate() {
		let x_offset = padding.left + index as u32 * spacing.horizontal;
		let bar_height = value as u32 * spacing.vertical / 100;
		if bar_height == 0 {
			continue;
		}
		draw_filled_rect_mut(
			canvas,
			Rect::at(
				(x_offset + 1) as i32,
				(height - padding.below - bar_height) as i32,
			)
			.of_size(spacing.horizontal - 1, bar_height),
			colour,
		);
	}
}

pub(crate) fn draw_graph_bars_with_gradient(
	canvas: &mut RgbImage,
	data: impl IntoIterator<Item = i32>,
	gradient: &MultiPointGradient,
	padding: Padding,
	spacing: Spacing,
) {
	let height = canvas.height();
	for (index, value) in data.into_iter().enumerate() {
		let x_offset = padding.left + index as u32 * spacing.horizontal;
		let bar_height = value as u32 * spacing.vertical / 100;
		if bar_height == 0 {
			continue;
		}
		draw_bar_with_gradient(
			canvas,
			Rect::at(
				(x_offset + 1) as i32,
				(height - padding.below - bar_height) as i32,
			)
			.of_size(spacing.horizontal - 1, bar_height),
			gradient,
		);
	}
}
