pub mod check;
pub mod infer;
pub mod r#type;

// enum Maybe<T> {
// 	Some(T),
// 	None,
// }

// trait Iterator<T> {
// 	fn next(&self) -> Maybe<T>;
// }

// struct VecIterator<T> {
// 	vec: Vec<T>,
// 	idx: u64,
// }

// trait Clone {}

// impl Clone for Maybe<u32> {}

// impl<T> Iterator<T> for VecIterator<T> {
// 	fn next(&self) -> Maybe<T> {
// 		Maybe::None
// 	}
// }

// trait IntoIterator<T, E>
// where
// 	T: Iterator<E>,
// {
// 	fn into_iter(&self) -> T;
// }

// impl IntoIterator<VecIterator<i32>, i32> for Vec<i32> {
// 	fn into_iter(&self) -> VecIterator<i32> {
// 		VecIterator {
// 			vec: self.to_vec(),
// 			idx: 0,
// 		}
// 	}
// }
