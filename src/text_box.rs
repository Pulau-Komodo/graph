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

struct LineWriter<'s> {
	lines: Vec<Vec<TextSegment<'s>>>,
	current_line_width: u32,
	can_break: bool,
}

impl<'s> LineWriter<'s> {
	fn new() -> Self {
		Self {
			lines: vec![Vec::new()],
			current_line_width: 0,
			can_break: true,
		}
	}
	fn current_line(&self) -> &Vec<TextSegment<'s>> {
		self.lines.last().unwrap()
	}
	fn current_line_mut(&mut self) -> &mut Vec<TextSegment<'s>> {
		self.lines.last_mut().unwrap()
	}
	fn trim_current_line(&mut self) {
		let trim = |line: &mut [TextSegment]| {
			let first = &mut line.first_mut().unwrap().text;
			if let Some(stripped) = first.strip_prefix(' ') {
				println!("Trimmed start off \"{stripped}\"");
				*first = stripped;
			}
			let last = &mut line.last_mut().unwrap().text;
			if let Some(stripped) = last.strip_suffix(' ') {
				println!("Trimmed end off \"{stripped}\"");
				*last = stripped;
			}
		};
		trim(self.current_line_mut());
	}
	fn new_line(&mut self) {
		self.trim_current_line();
		self.lines.push(Vec::new());
		self.current_line_width = 0;
	}
	fn add_to_current_line(&mut self, segment: TextSegment<'s>, width: u32) {
		//println!("Adding \"{}\"", segment.text);
		self.current_line_mut().push(segment);
		self.current_line_width += width;
	}
	fn add_segment(
		&mut self,
		segment: TextSegment<'s>,
		font: &FontRef<'_>,
		scale: PxScale,
		width: u32,
	) {
		let mut start_index = 0;
		let mut prev_break_point = None;
		for index in segment
			.text
			.char_indices()
			.filter_map(|(index, char)| (char == ' ').then_some(index))
		{
			self.can_break = true;
			let segment_width =
				drawing::text_size(scale, &font, &segment.text[start_index..index]).0;
			let line_width = self.current_line_width + segment_width;
			if line_width <= width {
				prev_break_point = Some(index);
				continue;
			}

			let end_index = prev_break_point.filter(|i| *i >= start_index).unwrap_or(index);
			let new_segment = &segment.text[start_index..end_index];
			if !new_segment.is_empty() {
				self.add_to_current_line(
					TextSegment {
						text: new_segment,
						color: segment.color,
					},
					0,
				);
			}
			self.new_line();
			prev_break_point = Some(index);
			start_index = end_index + 1;
			if start_index >= segment.text.len() {
				return;
			}
		}
		let remainder = &segment.text[start_index..];
		if !remainder.is_empty() {
			let segment_width = drawing::text_size(scale, &font, remainder).0;
			let line_width = self.current_line_width + segment_width;
			if line_width > width && self.can_break && !self.current_line().is_empty() {
				self.new_line();
			}
			self.add_to_current_line(
				TextSegment {
					text: remainder,
					color: segment.color,
				},
				segment_width,
			);
			if !remainder.ends_with(' ') {
				self.can_break = false;
			}
		}
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
		let mut lines = LineWriter::new();
		for segment in text {
			lines.add_segment(*segment, &font, font_scale, width);
		}

		if lines.current_line().is_empty() {
			lines.lines.pop();
		} else {
			lines.trim_current_line();
		}

		Self {
			lines: lines.lines,
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

		let segments = [
			TextSegment::new("Minimum", Rgb([0, 148, 255])),
			TextSegment::white(", "),
			TextSegment::new("maximum", Rgb([255, 0, 0])),
			TextSegment::white(" and "),
			TextSegment::new("apparent minimum and maximum", Rgb([0, 170, 33])),
			TextSegment::white(" temperatures (°C)"),
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

		let segments = [
			TextSegment::new("Minimum", Rgb([0, 148, 255])),
			TextSegment::white(", "),
			TextSegment::new("maximum", Rgb([255, 0, 0])),
			TextSegment::white(" and "),
			TextSegment::new("apparent minimum and maximum", Rgb([0, 170, 33])),
			TextSegment::white(" temperatures (°C)"),
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
	#[test]
	fn big_test() {
		let font_data: &[u8] = include_bytes!("../RobotoMono-Regular.ttf");
		let font = ab_glyph::FontRef::try_from_slice(font_data).expect("Failed to read font");

		let scale = PxScale { x: 15.0, y: 15.0 };
		let test_text = "The quick brown fox jumped over the lazy dog.";
		let char_width =
			drawing::text_size(scale, &font, test_text).0 as f32 / test_text.len() as f32;
		println!("char_width: {char_width}");

		let segments = [
			TextSegment::white("The quick "),
			TextSegment::new("brown", Rgb([150, 75, 0])),
			TextSegment::white(" fox jumped over the "),
			TextSegment::new("l", Rgb([200, 200, 255])),
			TextSegment::new("a", Rgb([100, 100, 255])),
			TextSegment::new("z", Rgb([200, 200, 255])),
			TextSegment::new("y", Rgb([100, 100, 255])),
			TextSegment::white(" dog."),
		];

		for i in 1..=45 {
			// if i != 3 {
			// 	continue;
			// }
			println!("{i}");
			let width = (i as f32 * char_width as f32) as u32;
			let text_box = TextBox::new(&segments, font.clone(), scale, width, 0);
			let mut chart = Chart::new(
				2,
				0,
				Spacing {
					horizontal: width,
					vertical: 0,
				},
				Padding {
					above: text_box.height(),
					below: 0,
					left: 0,
					right: 0,
				},
			);
			chart.draw(text_box);
			let canvas = chart.into_canvas();
			let _ = canvas.save(format!("./test-{i}.png"));
		}
	}
}
