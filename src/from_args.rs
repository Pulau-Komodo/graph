use itertools::Itertools;

pub trait FromArgs<const L: usize> {
	fn from_args(args: [String; L]) -> Self;
}

pub fn data_from_args<T: FromArgs<CHUNK_SIZE>, const CHUNK_SIZE: usize>(
	args: Vec<String>,
) -> Vec<T> {
	let mut data = Vec::with_capacity(args.len() / CHUNK_SIZE);
	for mut item in args.into_iter().chunks(CHUNK_SIZE).into_iter() {
		let elements = [(); CHUNK_SIZE].map(|_| {
			item.next()
				.unwrap_or_else(|| panic!("Arguments could not be grouped into {CHUNK_SIZE}s"))
		});
		data.push(T::from_args(elements));
	}
	data
}
