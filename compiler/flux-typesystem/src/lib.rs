pub mod check;
pub mod infer;
pub mod r#type;

// enum Maybe<T> {
// 	Some(T),
// 	None,
// }

// trait Iterator<T: Foo> {
// 	fn next(&self) -> Maybe<T>;
// }

// struct VecIterator<T> {
// 	vec: Vec<T>,
// 	idx: u64,
// }

// trait Foo {}

// impl<T: Foo> Iterator<T> for VecIterator<T> {
// 	fn next(&self) -> Maybe<T> {
// 		Maybe::None
// 	}
// }

// trait IntoIterator<T, E>
// where
// 	T: Iterator<E>,
// 	E: Foo,
// {
// 	fn into_iter(&self) -> T;
// }

// impl<T: Foo + Clone> IntoIterator<VecIterator<T>, T> for Vec<T> {
// 	fn into_iter(&self) -> VecIterator<T> {
// 		VecIterator {
// 			vec: self.to_vec(),
// 			idx: 0,
// 		}
// 	}
// }
