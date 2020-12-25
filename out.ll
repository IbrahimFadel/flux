; ModuleID = 'TheModule'
source_filename = "TheModule"

%string = type { i8*, i64, i64, i64 }

declare i32 @printf(i8*, ...)

define void @createString(%string* %str) {
entry:
  %str1 = alloca %string*, align 8
  store %string* %str, %string** %str1, align 8
  %str1_loaded = load %string*, %string** %str1, align 8
}
