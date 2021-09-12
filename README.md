# pi-lang

pi-lang is a statically typed programming language inspired by go.

Here's an example program:

```go

// Single line comment

/*
 * Multi-line comment
*/

// --- Type declarations ---
type Foo i32

type Animal interface {
	Hello() -> i32
}

type Dog struct {
	pub mut i32 Age
	pub i32 *X, Y, Z
	mut i32 Weight
	i32 Name
}

// -------------------------

// This method implements the Animal interface
fn (Dog *dog) Hello() -> i32 {
	return 0
}

/*
 * This is just a method
 * No return type specified = void
*/
fn (Dog *dog) Other() {
}

fn main() -> i32 {
	// Initialized to null
	Animal animal
	// This is really an i32
	Foo x = 0

	// Three mutable floats
	mut f32 a, b, c = 1, 2, 3

	return 0
}
```

As you can see, the way it works (interfaces + structs) is very similar to go.

These are the primitive types of the language:

`i64, u64, i32, u32, i16, u16, i8, u8, f64, f32, bool, string, void`

The compiler is still being rewritten, so certain functionality is not yet in the language.
