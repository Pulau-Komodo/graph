use ab_glyph::{FontRef, PxScale};
use image::Rgb;
use imageproc::drawing;

use crate::generic_graph::ChartElement;

#[derive(Debug, Clone, Copy)]
pub struct TextSegment<'s> {
	pub text: &'s str,
	pub color: Rgb<u8>,
}

impl<'s> TextSegment<'s> {
	pub fn new(text: &'s str, color: Rgb<u8>) -> Self {
		Self { text, color }
	}
	pub fn white(text: &'s str) -> Self {
		let color = Rgb([255, 255, 255]);
		Self { text, color }
	}
}

#[derive(Debug, Clone)]
pub struct TextBox<'f, 's> {
	lines: Vec<Vec<TextSegment<'s>>>,
	font: FontRef<'f>,
	font_scale: PxScale,
	line_distance: u32,
}

impl<'f, 's> TextBox<'f, 's> {
	/// This is mildly expensive to make, so consider reusing it.
	pub fn new(
		text: &[TextSegment<'s>],
		font: FontRef<'f>,
		font_scale: PxScale,
		width: u32,
		line_distance: u32,
	) -> Self {
		let mut lines = Vec::new();
		let mut current_line = Vec::new();
		let mut current_line_width = 0;
		for segment in text {
			let mut start_index = 0;
			let mut prev_space_index = 0;
			for index in segment
				.text
				.char_indices()
				.filter_map(|(index, char)| (char == ' ').then_some(index))
				.chain([segment.text.len()])
			{
				let segment_width =
					drawing::text_size(font_scale, &font, &segment.text[start_index..index]).0;
				let line_width = current_line_width + segment_width;
				if line_width > width {
					let new_segment = &segment.text[start_index..prev_space_index];
					if new_segment.is_empty() {
						if !current_line.is_empty() {
							lines.push(current_line);
							current_line = Vec::new();
							current_line_width = 0;
						}
						continue;
					}
					current_line.push(TextSegment {
						text: new_segment,
						color: segment.color,
					});
					lines.push(current_line);
					current_line = Vec::new();
					current_line_width = 0;
					start_index = prev_space_index + 1;
				} else {
					prev_space_index = index;
				}
			}
			let remainder = &segment.text[start_index..];
			if !remainder.is_empty() {
				let segment_width = drawing::text_size(font_scale, &font, remainder).0;
				current_line_width += segment_width;
				current_line.push(TextSegment {
					text: remainder,
					color: segment.color,
				});
			}
		}

		if !current_line.is_empty() {
			lines.push(current_line);
		}

		Self {
			lines,
			font,
			font_scale,
			line_distance,
		}
	}
	pub fn height(&self) -> u32 {
		self.lines.len() as u32 * self.font_scale.y as u32
			+ self.lines.len().saturating_sub(1) as u32 * self.line_distance
	}
}

impl ChartElement for TextBox<'_, '_> {
	fn draw(self, chart: &mut crate::generic_graph::Chart) {
		let mut cursor_y = self.line_distance as i32;
		for line in self.lines {
			let mut cursor_x = chart.padding.left as i32;
			for segment in line {
				imageproc::drawing::draw_text_mut(
					&mut chart.canvas,
					segment.color,
					cursor_x,
					cursor_y,
					self.font_scale,
					&self.font,
					segment.text,
				);
				let (text_width, _text_height) =
					imageproc::drawing::text_size(self.font_scale, &self.font, segment.text);
				cursor_x += text_width as i32;
			}
			cursor_y += self.font_scale.y as i32 + self.line_distance as i32;
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		drawing::{Padding, Spacing},
		generic_graph::Chart,
	};

	use super::*;

	#[test]
	fn test_line_wrap() {
		let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
		let font = ab_glyph::FontRef::try_from_slice(font_data).expect("Failed to read font");

		let segments = vec![
			TextSegment {
				text: "Minimum",
				color: Rgb([0, 148, 255]),
			},
			TextSegment {
				text: ", ",
				color: Rgb([255, 255, 255]),
			},
			TextSegment {
				text: "maximum",
				color: Rgb([255, 0, 0]),
			},
			TextSegment {
				text: " and ",
				color: Rgb([255, 255, 255]),
			},
			TextSegment {
				text: "apparent minimum and maximum",
				color: Rgb([0, 170, 33]),
			},
			TextSegment {
				text: " temperatures (°C)",
				color: Rgb([255, 255, 255]),
			},
		];
		let text_box = TextBox::new(&segments, font, PxScale { x: 15.0, y: 15.0 }, 151, 5);
		for line in text_box.lines {
			println!("{:?}", line);
		}
	}
	#[test]
	fn test_drawing() {
		let font_data: &[u8] = include_bytes!("../RobotoCondensed-Regular.ttf");
		let font = ab_glyph::FontRef::try_from_slice(font_data).expect("Failed to read font");

		let segments = vec![
			TextSegment {
				text: "Minimum",
				color: Rgb([0, 148, 255]),
			},
			TextSegment {
				text: ", ",
				color: Rgb([255, 255, 255]),
			},
			TextSegment {
				text: "maximum",
				color: Rgb([255, 0, 0]),
			},
			TextSegment {
				text: " and ",
				color: Rgb([255, 255, 255]),
			},
			TextSegment {
				text: "apparent minimum and maximum",
				color: Rgb([0, 170, 33]),
			},
			TextSegment {
				text: " temperatures (°C)",
				color: Rgb([255, 255, 255]),
			},
		];
		let text_box = TextBox::new(&segments, font, PxScale { x: 15.0, y: 15.0 }, 151, 5);
		let mut chart = Chart::new(
			7,
			0,
			Spacing {
				horizontal: 25,
				vertical: 3,
			},
			Padding {
				above: 3 + text_box.height(),
				below: 19,
				left: 21,
				right: 9,
			},
		);
		chart.draw(text_box);
		let canvas = chart.into_canvas();
		let _ = canvas.save("./test.png");
	}
}
