# Sandscript

Sandscript is a small compiled, statically typed language. It's not very useful, but it's been fun working on.

Rewriting in yet again -- and hey, there's new syntax.

The syntax in sandscript is very similar to rust:

```cpp
i8 globvar = 10 + 5;
i32 othernum = 25;

fn sum(i32 num1, i32 num2) -> i8 {
    i8 retval = toi8(num1) + toi8(num2);
    return retval;
}

fn main() -> i8 {
    i32 x = 20;
    i32 y = 30;
    i8 mysum = sum(x, y);
    i8 final = toi8(othernum) + globvar * mysum;
    return final;
}
```

This gets converted into LLVM IR:

```llvm
; ModuleID = 'Module'
source_filename = "Module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@globvar = common global i8 0
@othernum = common global i32 0

define i8 @sum(i32 %num1, i32 %num2) {
entry:
  %num22 = alloca i32, align 4
  %num11 = alloca i32, align 4
  store i32 %num1, i32* %num11, align 4
  store i32 %num2, i32* %num22, align 4
  %0 = trunc i32 %num1 to i8
  %1 = trunc i32 %num2 to i8
  %addtmp = add i8 %1, %0
  ret i8 %addtmp
}

define void @__assign_global_variables() {
entry:
  store i8 15, i8* @globvar, align 1
  store i32 25, i32* @othernum, align 4
  ret void
}

define i8 @main() {
entry:
  call void @__assign_global_variables()
  %calltmp = call i8 @sum(i32 20, i32 30)
  %0 = load i8, i8* @globvar, align 1
  %multmp = mul i8 %0, %calltmp
  %1 = load i8, i8* bitcast (i32* @othernum to i8*), align 4
  %addtmp = add i8 %1, %multmp
  ret i8 %addtmp
}

; Function Attrs: nounwind
declare void @llvm.stackprotector(i8*, i8**) #0

attributes #0 = { nounwind }
```

And that gets compiled
