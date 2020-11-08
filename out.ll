; ModuleID = 'Module'
source_filename = "Module"

define i32 @main() {
entry:
  %num1 = alloca i32
  store i32 230, i32* %num1
  %num2 = alloca i32
  store i32 220, i32* %num2
  %num3 = alloca i32
  store i32 230, i32* %num3
  %num4 = alloca i32
  store i32 13, i32* %num4
  %ifcond = icmp eq i32* %num1, %num2
  %ifcond1 = icmp eq i32* %num3, %num4
  %ifcond2 = icmp eq i32* %num1, %num3
  %0 = and i1 %ifcond, %ifcond1
  %1 = or i1 %0, %ifcond2
  %2 = icmp eq i1 %1, true
  br i1 %2, label %then, label %else

then:                                             ; preds = %entry
  %x = alloca i32
  store i32 10, i32* %x
  %3 = load i32, i32* %x
  ret i32 %3
  br label %continue

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %then
  ret i32 0
}
