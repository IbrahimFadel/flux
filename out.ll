; ModuleID = 'Module'
source_filename = "Module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

define i32 @add(i32 %num) {
entry:
  %num1 = alloca i32
  store i32 %num, i32* %num1
  %test = alloca i32
  store i32 30, i32* %test
  %test2 = load i32, i32* %test
  %num13 = load i32, i32* %num1
  %addtmp = add i32 %test2, %num13
  ret i32 %addtmp
}

define i32 @main() {
entry:
  %res = alloca i32
  %calltmp = call i32 @add(i32 12)
  store i32 %calltmp, i32* %res
  %res1 = load i32, i32* %res
  ret i32 %res1
}

; Function Attrs: nounwind
declare void @llvm.stackprotector(i8*, i8**) #0

attributes #0 = { nounwind }
