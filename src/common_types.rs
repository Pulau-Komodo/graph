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
