; ModuleID = 'main.ss'
source_filename = "main.ss"

declare i32 @bytesFunction()

declare i32 @ioFunction()

define void @main() {
entry:
  %0 = call i32 @ioFunction()
  ret void
}
