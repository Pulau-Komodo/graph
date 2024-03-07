use itertools::Itertools;
use oklab::{oklab_to_srgb, srgb_to_oklab, Oklab, RGB};

#[derive(Debug, Clone, Copy)]
pub struct Point<T: std::ops::Add + std::ops::Sub> {
	pub x: T,
	pub y: T,
}

#[derive(Debug, Clone, Copy)]
pub struct Range<T: std::ops::Sub<Output = T> + Ord + Copy> {
	start: T,
	end: T,
}

impl<T: std::ops::Sub<Output = T> + Ord + Copy> Range<T> {
	pub fn new(start: T, end: T) -> Self {
		if end < start {
			panic!("End of range is before start of range")
		} else {
			Self { start, end }
		}
	}
	pub fn len(&self) -> T {
		self.end - self.start
	}
	pub fn start(&self) -> T {
		self.start
	}
	pub fn end(&self) -> T {
		self.end
	}
}

#[derive(Debug)]
pub struct GradientPoint {
	/// The point in the gradient where it should be this colour
	point: u32,
	/// The colour it should be in oklab
	colour: Oklab,
}

impl GradientPoint {
	pub fn from_rgb(point: u32, [r, g, b]: [u8; 3]) -> Self {
		let colour = srgb_to_oklab(RGB::new(r, g, b));
		Self { point, colour }
	}
	pub fn point(&self) -> u32 {
		self.point
	}
}

pub struct MultiPointGradient {
	points: Vec<GradientPoint>,
}

impl MultiPointGradient {
	pub fn new(points: Vec<GradientPoint>) -> Self {
		if points
			.iter()
			.tuple_windows()
			.any(|(a, b)| a.point() >= b.point())
		{
			panic!("Gradient points not in increasing order");
		}
		Self { points }
	}
	pub fn get_colour(&self, point: u32) -> [u8; 3] {
		let (start, end) = self
			.points
			.iter()
			.tuple_windows()
			.find_or_last(|(_start, end)| point <= end.point())
			.unwrap();
		let adjusted_point = (point - start.point()) as f32 / (end.point() - start.point()) as f32;
		let adjusted_point = adjusted_point.min(1.0).max(0.0);
		/*println!(
			"{point} adjusted to {adjusted_point} between {:?} and {:?}",
			start, end
		);*/
		let colour = Oklab {
			l: between_point(start.colour.l, end.colour.l, adjusted_point),
			a: between_point(start.colour.a, end.colour.a, adjusted_point),
			b: between_point(start.colour.b, end.colour.b, adjusted_point),
		};
		let rgb = oklab_to_srgb(colour);
		[rgb.r, rgb.g, rgb.b]
	}
}

fn between_point(start: f32, end: f32, point: f32) -> f32 {
	if start < end {
		start + point * (end - start)
	} else {
		start - point * (start - end)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn gradient() {
		let gradient = MultiPointGradient::new(vec![
			GradientPoint::from_rgb(0, [0, 0, 0]),
			GradientPoint::from_rgb(100, [255, 255, 255]),
		]);
		for n in (0..=100).step_by(10) {
			println!("{:?}", gradient.get_colour(n));
		}
	}

	#[test]
	fn multi_point_gradient() {
		let gradient = MultiPointGradient::new(vec![
			GradientPoint::from_rgb(0, [0, 0, 0]),
			GradientPoint::from_rgb(100, [255, 255, 255]),
			GradientPoint::from_rgb(200, [0, 0, 0]),
			GradientPoint::from_rgb(300, [255, 255, 255]),
		]);
		for n in (0..=300).step_by(10) {
			println!("{:?}", gradient.get_colour(n))
		}
	}
}
