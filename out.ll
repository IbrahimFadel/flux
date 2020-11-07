; ModuleID = 'Module'
source_filename = "Module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@globvar = common global i8 0

define void @__assign_global_variables() {
entry:
  store i8 15, i8* @globvar, align 1
  ret void
}

define i8 @main() {
entry:
  call void @__assign_global_variables()
  %0 = load i8, i8* @globvar, align 1
  ret i8 %0
}

; Function Attrs: nounwind
declare void @llvm.stackprotector(i8*, i8**) #0

attributes #0 = { nounwind }
