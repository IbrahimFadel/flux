# Sandscript

Sandscript is a small compiled, statically typed language. It's not very useful, but it's been fun working on.

Rewriting in yet again -- and hey, there's new syntax.

The syntax in sandscript is very similar to rust:

```cpp
fn add() -> int {
    return 5;
}

fn main(int hi, int hello) -> int {
    int y = 10 * 3 + 43;
    int x = y + 5;

    int res = add();

    return y + x * res;
}
```

This gets converted into LLVM IR:

```llvm
define i32 @add() {
entry:
  ret i32 5
}

define i32 @main(i32 %hi, i32 %hello) {
entry:
  %y = alloca i32
  store i32 73, i32* %y
  %y1 = load i32, i32* %y
  %x = alloca i32
  %addtmp = add i32 %y1, 5
  store i32 %addtmp, i32* %x
  %x2 = load i32, i32* %x
  %res = alloca i32
  %calltmp = call i32 @add()
  store i32 %calltmp, i32* %res
  %res3 = load i32, i32* %res
  %addtmp4 = mul i32 %x2, %res3
  %addtmp5 = add i32 %y1, %addtmp4
  ret i32 %addtmp5
}
```

And that gets optimized and compiled to a binary