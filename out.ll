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
