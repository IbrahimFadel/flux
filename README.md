# &#120571; Flux

Flux is a statically typed programming language inspired by go and rust.

The compiler is currently being rewritten, but here is the general structure of the project:

1. Generate Concrete Syntax Tree with the help of [logos](https://github.com/maciejhirsz/logos) and [rowan](https://github.com/rust-analyzer/rowan)
2. Transform that CST into a High-Level IR (HIR)
   * In the future, the (extremely tedious to write) getters and implementations needed for this step will be generated automatically -- see `flux-syntax/src/builder.rs` for the first attempt. This goal, however, has been put on the back burner for now as it's not very important in the grand scheme of things and seems to be somewhat complicated in certain cases.
3. Typecheck the CST
   * At this stage, flux will figure out all sorts of things about the program such as the exact types of integers (`i32`, or `u12`?), what structs implement what interfaces, and other things like that.
   * This is the final stage when errors are caught and dealt with properly. If an error is encountered after typechecking, it is considered an internal compiler error, and flux will panic.
4. Lower HIR to Middle IR
   * The first ever implementation of the flux MIR was essentially a slightly modified version of LLVM IR. While this is fine and made code generation significantly easier later on, future plans for flux include making the MIR an RVSDG.
5. So far, none have been implemented, but optimizations will be run on the MIR
6. Lower MIR to LLVM IR
7. Emit object files

## Why use a CST?

If you take a look at the codebase, or even the previous section, you'll notice that the use of a CST requires some really annoying "ast" code where getters need to be made for each node (ex. functions in the CST need `get_return_type`, `get_params`, and `get_body` methods).

Ignoring this slight hiccup, [CSTs have a lot of advantages](https://rdambrosio016.github.io/rust/2020/09/18/pure-ast-based-linting-sucks.html). While, to me, traversal is a compelling enough reason, the main reason I am using a CST now is because I hope to make a linter/formatter for flux down the road, and CSTs will make my life a lot easier when I get to that stage.

## LLVM & Self Hosting

Currently I am targetting LLVM IR. This is almost completely for convenience's sake as code generation without LLVM is a nightmare. BUT, where's the fun in taking the easy way out? In the future, I want to eliminate the LLVM dependency. Once that is done, i'm hoping to self host flux. That would be cool.

## Example Program:

Instead of taking the time to write out a fully fledged introduction to the language that is bound to be completely different by the time i finish writing it, i'll just leave this:

```go
// --- Type declarations ---
type Foo i32

type Animal interface {
	fn do_animal_stuff(mut this);
	pub fn age(this) -> u8;
}

type Dog struct {
	u8 age;
	pub f32 x;
	mut u6 weight; // arbitrary length integers
}

// implement the Animal interface
apply Animal to Dog {
	fn do_animal_stuff(mut this) {
		this.weight = 2;
	}

	pub fn age(this) -> u8 {
		return this.age;
	}
}

// Add methods to Dog
apply Dog {
	pub fn do_dog_stuff(this) {
		/* 
			a method only dog's can do
		*/
	}
}

fn main() -> i32 {
	Dog d = Dog {
		age: 0,
		x: 1.0,
		weight: 5,
	};

	u8 dog_age = d.age();
	d.do_dog_stuff();

	return 0;
}
```