; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @add(i32 %num1, i32 %num2) {
entry:
  %addtmp = add i32 %num2, %num1
  ret i32 %addtmp
}

define i32 @main() {
entry:
  %calltmp = call i32 @add(i32 1, i32 2)
  ret i32 %calltmp
}
