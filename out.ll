; ModuleID = 'TheModule'
source_filename = "TheModule"

%string = type { i8*, i64, i64, i64 }

declare i32 @printf(i8*, ...)

define void @createString(%string* %str) {
entry:
  %str1 = alloca %string*, align 8
  store %string* %str, %string** %str1, align 8
  %str1_loaded = load %string*, %string** %str1, align 8
  %0 = load %string*, %string** %str1, align 8
  %_length = getelementptr %string, %string* %0, i32 2
  store i64 0, %string* %_length, align 4
  %1 = load %string*, %string** %str1, align 8
  %_length2 = getelementptr %string, %string* %1, i32 2
  %_length2_loaded = load %string, %string* %_length2, align 8
  %2 = load %string*, %string** %str1, align 8
  %_maxLength = getelementptr %string, %string* %2, i32 3
  store i64 0, %string* %_maxLength, align 4
  %3 = load %string*, %string** %str1, align 8
  %_maxLength3 = getelementptr %string, %string* %3, i32 3
  %_maxLength3_loaded = load %string, %string* %_maxLength3, align 8
  %4 = load %string*, %string** %str1, align 8
  %_factor = getelementptr %string, %string* %4, i32 1
  store i64 16, %string* %_factor, align 4
  %5 = load %string*, %string** %str1, align 8
  %_factor4 = getelementptr %string, %string* %5, i32 1
  %_factor4_loaded = load %string, %string* %_factor4, align 8
  ret void
}

define i32 @main() {
entry:
  %testString = alloca %string, align 8
  %testString_loaded = load %string, %string* %testString, align 8
  %calltmp = call void @createString(%string %testString_loaded)
  ret i32 0
}
