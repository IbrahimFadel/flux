; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @main() {
entry:
  %x = alloca i32, align 4
  store i32 0, i32* %x, align 4
  %i = alloca i32, align 4
  store i32 0, i32* %i, align 4
  %i_loaded = load i32, i32* %i, align 4
  %lttmp = icmp slt i32 %i_loaded, 100
  br i1 %lttmp, label %loop, label %continue

loop:                                             ; preds = %continue2, %entry
  %x_loaded = load i32, i32* %x, align 4
  %ifcond = icmp eq i32 %x_loaded, 33
  br i1 %ifcond, label %then, label %else

continue:                                         ; preds = %continue2, %entry
  %x_loaded8 = load i32, i32* %x, align 4
  ret i32 %x_loaded8

then:                                             ; preds = %loop
  %x_loaded1 = load i32, i32* %x, align 4
  ret i32 %x_loaded1
  br label %continue2

else:                                             ; preds = %loop
  br label %continue2

continue2:                                        ; preds = %else, %then
  %x_loaded3 = load i32, i32* %x, align 4
  %addtmp = add i32 %x_loaded3, 1
  store i32 %addtmp, i32* %x, align 4
  %i_loaded4 = load i32, i32* %i, align 4
  %addtmp5 = add i32 %i_loaded4, 1
  store i32 %addtmp5, i32* %i, align 4
  %i_loaded6 = load i32, i32* %i, align 4
  %lttmp7 = icmp slt i32 %i_loaded6, 100
  br i1 %lttmp7, label %loop, label %continue
}
