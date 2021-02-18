; ModuleID = '/home/ibrahim/dev/sandscriptold/main.ss'
source_filename = "/home/ibrahim/dev/sandscriptold/main.ss"

%string = type { i8 }

declare void @free(i8*)

declare i8* @malloc(i64)

declare i8* @memcpy(i8*, i8*, i64)

declare i32 @printf(i8*)

define i32 @main() {
entry:
  %foo = alloca %string, align 8
  %foo1 = load %string, %string* %foo, align 1
  %test = getelementptr inbounds %string, %string* %foo, i32 0, i32 0
  %0 = load i8, i8* %test, align 1
  store i8 33, i8* %test, align 1
  ret i32 0
}
