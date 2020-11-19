; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @main() {
entry:
  %x = alloca i32, align 4
  store i32 6, i32* %x, align 4
  %0 = load i32, i32* %x, align 4
  %z = alloca i32*, align 8
  store i32* %x, i32** %z, align 8
  %z_val = alloca i32, align 4
  %1 = load i32*, i32** %z, align 8
  %2 = load i32, i32* %1, align 4
  store i32 %2, i32* %z_val, align 4
  %3 = load i32, i32* %z_val, align 4
  %final = alloca i32, align 4
  store i32 %3, i32* %final, align 4
  %4 = load i32, i32* %final, align 4
  %ifcond = icmp eq i32 %4, 5
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %entry
  ret i32 2
  br label %continue

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %then
  ret i32 %4
}
