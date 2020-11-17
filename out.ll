; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @main() {
entry:
  %x = alloca i32, align 4
  store i32 5, i32* %x, align 4
  %0 = load i32, i32* %x, align 4
  %ifcond = icmp eq i32 %0, 5
  br i1 %ifcond, label %then, label %else3

then:                                             ; preds = %entry
  %y = alloca i32, align 4
  store i32 5, i32* %y, align 4
  %1 = load i32, i32* %y, align 4
  %ifcond1 = icmp eq i32 %0, %1
  br i1 %ifcond1, label %then2, label %else

then2:                                            ; preds = %then
  ret i32 5
  br label %continue

else:                                             ; preds = %then
  br label %continue

continue:                                         ; preds = %else, %then2
  ret i32 2
  br label %continue4

else3:                                            ; preds = %entry
  br label %continue4

continue4:                                        ; preds = %else3, %continue
  ret i32 0
}
