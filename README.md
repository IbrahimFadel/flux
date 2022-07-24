# &#120651; Flux

Flux is a statically typed, ahead of time compiled programming language mainly inspired by rust.

The compiler is still *very much* a work in progress, but here is the general structure of the project:

1. Generate Concrete Syntax Tree with the help of [logos](https://github.com/maciejhirsz/logos) and [rowan](https://github.com/rust-analyzer/rowan)
2. Transform that CST into a High-Level IR (HIR)
	* Typechecking is done simultaneously, and roughly follows a Hindley-Milner type system (*disclaimer*: i'm still confused as to what exactly qualifies as HM...)
4. Lower HIR to Middle IR
5. Optimize MIR
6. Lower MIR to LLVM IR
7. Emit object files

## Example Program:

Syntax is subject to change -- if you want up to date syntax, your best bet is to just check `examples/main.flx` because that's the file i tend to use when testing whatever i'm working on.

```rust
type Vec struct<T> {
	len u64,
	capacity u64,
	buf *T,
}

apply<T> to Vec<T> {
	pub fn new() -> Vec<T> => {
		let initial_capacity = 32;
		Vec {
			len: 0,
			capacity: 0,
			buf: @flux.intrinsics.malloc(initial_capacity),
		}
	}

	fn delete() => {
		if self.buf != @flux.intrinsics.nullptr {
			@flux.intrinsics.free(self.buf);
		};
	}

	fn resize(u64 value) => {
		let output = @flux.intrinsics.malloc(value);
		if output == @flux.intrinsics.nullptr {
			return;
		};
		@flux.intrinsics.memcpy(output, self.buf, self.len);
		@flux.intrinsics.free(self.buf);
		self.buf = output;
		self.capacity = value;
	}

	fn push(T value) => {
		if self.len == self.capacity {
			let new_capacity = self.capacity * 2;
			self.resize(new_capacity);
		};
		self.buf[self.len] = value;
		self.len = self.len + 1;
	}
}
```