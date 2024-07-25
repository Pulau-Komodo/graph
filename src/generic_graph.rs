use ab_glyph::{FontRef, PxScale};
use image::RgbImage;

pub use image::Rgb;

use crate::{
	colours,
	common_types::{MultiPointGradient, Range},
	drawing::{
		draw_graph_bars, draw_graph_bars_with_gradient, draw_graph_lines, draw_outer_lines,
		fill_canvas, horizontal_lines_and_labels, vertical_lines_and_labels, MarkIntervals,
		Padding, Spacing,
	},
};

pub struct Chart {
	canvas: RgbImage,
	padding: Padding,
	spacing: Spacing,
}

impl Chart {
	pub fn new(data_len: usize, data_range: u32, spacing: Spacing, padding: Padding) -> Self {
		let width = (data_len as u32 - 1) * spacing.horizontal + padding.horizontal();
		let height = data_range * spacing.vertical / 100 + padding.vertical();
		let mut canvas = RgbImage::new(width, height);
		fill_canvas(&mut canvas, colours::BACKGROUND);
		Self {
			canvas,
			padding,
			spacing,
		}
	}
	pub fn draw<E: ChartElement>(&mut self, element: E) {
		element.draw(self);
	}
	pub fn into_canvas(self) -> RgbImage {
		self.canvas
	}
}

pub trait ChartElement {
	fn draw(self, chart: &mut Chart);
}

pub struct AxisGridLabels<H>
where
	H: Iterator<Item = u8>,
{
	pub vertical_intervals: MarkIntervals,
	pub horizontal_intervals: MarkIntervals,
	pub vertical_label_range: Range<i32>,
	pub horizontal_labels: H,
	pub horizontal_labels_centered: bool,
	pub font: FontRef<'static>,
	pub font_scale: PxScale,
}

impl<H> ChartElement for AxisGridLabels<H>
where
	H: Iterator<Item = u8>,
{
	fn draw(self, chart: &mut Chart) {
		draw_outer_lines(&mut chart.canvas, chart.padding);
		vertical_lines_and_labels(
			&mut chart.canvas,
			self.horizontal_labels,
			self.horizontal_intervals,
			&self.font,
			self.font_scale,
			chart.padding,
			chart.spacing.horizontal,
			self.horizontal_labels_centered,
		);
		horizontal_lines_and_labels(
			&mut chart.canvas,
			self.vertical_label_range,
			self.vertical_intervals,
			&self.font,
			self.font_scale,
			chart.padding,
			chart.spacing.vertical,
		);
	}
}

pub struct GradientBars<D>
where
	D: Iterator<Item = i32>,
{
	gradient: MultiPointGradient,
	data: D,
}

impl<D> ChartElement for GradientBars<D>
where
	D: Iterator<Item = i32>,
{
	fn draw(self, chart: &mut Chart) {
		draw_graph_bars_with_gradient(
			&mut chart.canvas,
			self.data,
			&self.gradient,
			chart.padding,
			chart.spacing,
		);
	}
}

pub struct SolidBars<D>
where
	D: Iterator<Item = i32>,
{
	pub colour: Rgb<u8>,
	pub data: D,
}

impl<D> ChartElement for SolidBars<D>
where
	D: Iterator<Item = i32>,
{
	fn draw(self, chart: &mut Chart) {
		draw_graph_bars(
			&mut chart.canvas,
			self.data,
			self.colour,
			chart.padding,
			chart.spacing,
		);
	}
}

pub struct Line<D>
where
	D: Iterator<Item = i32>,
{
	pub colour: Rgb<u8>,
	pub data: D,
	pub max: i32,
}

impl<D> ChartElement for Line<D>
where
	D: Iterator<Item = i32>,
{
	fn draw(self, chart: &mut Chart) {
		draw_graph_lines(
			&mut chart.canvas,
			self.data,
			self.colour,
			self.max,
			chart.padding,
			chart.spacing,
		);
	}
}
