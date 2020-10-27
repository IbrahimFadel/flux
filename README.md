# Sandscript

Sandscript is a small compiled, statically typed language. It's not very useful, but it's been fun working on.

Rewriting in yet again -- and hey, there's new syntax.

The syntax in sandscript is very similar to rust:

```cpp
fn sum(i32 num1, i32 num2) -> i8 {
    i32 sum = num1 + num2;
    i8 tst = toi8(sum);
    return tst;
}

fn main() -> i8 {
    i32 x = 10;
    i32 y = 4;

    i8 res = sum(x, y);

    return res;
}
```

This gets converted into LLVM IR:

```llvm
; ModuleID = 'Module'
source_filename = "Module"

define i8 @sum(i32 %num1, i32 %num2) {
entry:
  %sum = alloca i32, align 4
  %addtmp = add i32 %num2, %num1
  store i32 %addtmp, i32* %sum, align 4
  %0 = bitcast i32* %sum to i8*
  %1 = trunc i32 %addtmp to i8
  ret i8 %1
}

define i8 @main() {
entry:
  %calltmp = call i8 @sum(i32 10, i32 4)
  ret i8 %calltmp
}
```

And that gets optimized and compiled to a binary
