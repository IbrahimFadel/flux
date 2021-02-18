; ModuleID = '/home/ibrahim/dev/sandscriptold/main.ss'
source_filename = "/home/ibrahim/dev/sandscriptold/main.ss"

%string = type { i8*, i64, i64, i64 }

declare void @free(i8*)

declare i8* @malloc(i64)

declare i8* @memcpy(i8*, i8*, i64)

declare i32 @printf(i8*)

declare void @string_create_default(%string*)

define i32 @main() {
entry:
  %0 = alloca %string, align 8
  call void @string_create_default(%string* %0)
  %1 = load %string, %string* %0, align 8
  ret i32 0
}
