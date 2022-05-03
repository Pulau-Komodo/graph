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
	/// The point in the gradient, from 0 to 1, where it should be this colour
	point: f32,
	/// The colour it should be in oklab
	colour: Oklab,
}

impl GradientPoint {
	pub fn from_rgb(point: f32, [r, g, b]: [u8; 3]) -> Self {
		if !(0.0..=1.0).contains(&point) {
			panic!("Gradient point out of range ({point} out of 0-1)");
		}
		let colour = srgb_to_oklab(RGB::new(r, g, b));
		Self { point, colour }
	}
	pub fn point(&self) -> f32 {
		self.point
	}
}

pub struct MultiPointGradient(Vec<GradientPoint>);

impl MultiPointGradient {
	pub fn new(points: Vec<GradientPoint>) -> Self {
		Self(points)
	}
	pub fn get_colour(&self, point: f32) -> [u8; 3] {
		let (start, end) = self
			.0
			.iter()
			.tuple_windows()
			.find_or_last(|(_start, end)| point <= end.point())
			.unwrap();
		let adjusted_point = (point - start.point()) / (end.point() - start.point());
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
			GradientPoint::from_rgb(0.0, [0, 0, 0]),
			GradientPoint::from_rgb(1.0, [255, 255, 255]),
		]);
		for n in 0..10 {
			println!("{:?}", gradient.get_colour(n as f32 / 9.0))
		}
	}

	#[test]
	fn multi_point_gradient() {
		let gradient = MultiPointGradient::new(vec![
			GradientPoint::from_rgb(0.0 / 9.0, [0, 0, 0]),
			GradientPoint::from_rgb(3.0 / 9.0, [255, 255, 255]),
			GradientPoint::from_rgb(6.0 / 9.0, [0, 0, 0]),
			GradientPoint::from_rgb(9.0 / 9.0, [255, 255, 255]),
		]);
		for n in 0..=20 {
			println!("{:?}", gradient.get_colour(n as f32 / 20.0))
		}
	}
}
