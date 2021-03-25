; ModuleID = '/home/ibrahim/dev/sandscript/main.ss'
source_filename = "/home/ibrahim/dev/sandscript/main.ss"

define i32 @test(i32 %x, i16 %y) {
entry:
  %x1 = alloca i32, align 4
  store i32 %x, i32* %x1, align 4
  %0 = load i32, i32* %x1, align 4
  %y2 = alloca i16, align 2
  store i16 %y, i16* %y2, align 2
  %1 = load i16, i16* %y2, align 2
  %2 = sext i16 %1 to i32
  %3 = add i32 %0, %2
  ret i32 %3
}

define i32 @main() {
entry:
  %0 = alloca i32, align 4
  store i32 0, i32* %0, align 4
  %x = load i32, i32* %0, align 4
  %1 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  %i = load i32, i32* %1, align 4
  br label %for.loop

for.loop:                                         ; preds = %for.loop, %entry
  %2 = load i32, i32* %1, align 4
  store i32 %2, i32* %0, align 4
  %3 = load i32, i32* %0, align 4
  %4 = add i32 %2, 1
  store i32 %4, i32* %1, align 4
  %5 = load i32, i32* %1, align 4
  %6 = icmp slt i32 %5, 20
  br i1 %6, label %for.loop, label %for.merge

for.merge:                                        ; preds = %for.loop
  %7 = alloca i32, align 4
  %8 = call i32 @test(i32 5, i16 10)
  store i32 %8, i32* %7, align 4
  %res = load i32, i32* %7, align 4
  %9 = alloca i32, align 4
  %10 = add i32 %res, %3
  store i32 %10, i32* %9, align 4
  %final = load i32, i32* %9, align 4
  ret i32 %3
}
