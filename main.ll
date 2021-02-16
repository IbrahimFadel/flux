; ModuleID = 'string.ss'
source_filename = "string.ss"

%string = type { i8*, i64, i64, i64 }

declare void @free(i8*)

declare i8* @malloc(i64)

declare i8* @memcpy(i8*, i8*, i64)

define void @string_create_default(%string* %this) {
entry:
  %0 = alloca %string*, align 8
  store %string* %this, %string** %0, align 8
  %1 = load %string*, %string** %0, align 8
  %buffer = getelementptr inbounds %string, %string* %1, i32 0, i32 0
  %2 = load i8*, i8** %buffer, align 8
  store i8* null, i8** %buffer, align 8
  %length = getelementptr inbounds %string, %string* %1, i32 0, i32 2
  %3 = load i64, i64* %length, align 4
  store i64 0, i64* %length, align 4
  %maxLength = getelementptr inbounds %string, %string* %1, i32 0, i32 3
  %4 = load i64, i64* %maxLength, align 4
  store i64 0, i64* %maxLength, align 4
  %factor = getelementptr inbounds %string, %string* %1, i32 0, i32 1
  %5 = load i64, i64* %factor, align 4
  store i64 16, i64* %factor, align 4
  ret void
}

define void @string_delete(%string* %this) {
entry:
  %0 = alloca %string*, align 8
  store %string* %this, %string** %0, align 8
  %1 = load %string*, %string** %0, align 8
  %buf = alloca i8*, align 8
  %buffer = getelementptr inbounds %string, %string* %1, i32 0, i32 0
  %2 = load i8*, i8** %buffer, align 8
  store i8* %2, i8** %buf, align 8
  %buf1 = load i8*, i8** %buf, align 8
  %3 = icmp ne i8* %buf1, null
  br i1 %3, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  call void @free(i8* %buf1)
  br label %if.merge

if.else:                                          ; preds = %entry
  br label %if.merge

if.merge:                                         ; preds = %if.else, %if.then
  ret void
}

define void @string_resize(%string* %this, i64 %value) {
entry:
  %0 = alloca i64, align 8
  %1 = alloca %string*, align 8
  store %string* %this, %string** %1, align 8
  %2 = load %string*, %string** %1, align 8
  store i64 %value, i64* %0, align 4
  %3 = load i64, i64* %0, align 4
  %output = alloca i8*, align 8
  %4 = call i8* @malloc(i64 %3)
  store i8* %4, i8** %output, align 8
  %output1 = load i8*, i8** %output, align 8
  %buf = alloca i8*, align 8
  %buffer = getelementptr inbounds %string, %string* %2, i32 0, i32 0
  %5 = load i8*, i8** %buffer, align 8
  store i8* %5, i8** %buf, align 8
  %buf2 = load i8*, i8** %buf, align 8
  %len = alloca i64, align 8
  %length = getelementptr inbounds %string, %string* %2, i32 0, i32 2
  %6 = load i64, i64* %length, align 4
  store i64 %6, i64* %len, align 4
  %len3 = load i64, i64* %len, align 4
  %7 = call i8* @memcpy(i8* %output1, i8* %buf2, i64 %len3)
  call void @free(i8* %buf2)
  %buffer4 = getelementptr inbounds %string, %string* %2, i32 0, i32 0
  %8 = load i8*, i8** %buffer4, align 8
  store i8* %output1, i8** %buffer4, align 8
  ret void
}

define void @string_add_char(%string* %this, i8 %value) {
entry:
  %0 = alloca i8, align 1
  %1 = alloca %string*, align 8
  store %string* %this, %string** %1, align 8
  %2 = load %string*, %string** %1, align 8
  store i8 %value, i8* %0, align 1
  %3 = load i8, i8* %0, align 1
  %len = alloca i64, align 8
  %length = getelementptr inbounds %string, %string* %2, i32 0, i32 2
  %4 = load i64, i64* %length, align 4
  store i64 %4, i64* %len, align 4
  %len1 = load i64, i64* %len, align 4
  %maxLen = alloca i64, align 8
  %maxLength = getelementptr inbounds %string, %string* %2, i32 0, i32 3
  %5 = load i64, i64* %maxLength, align 4
  store i64 %5, i64* %maxLen, align 4
  %maxLen2 = load i64, i64* %maxLen, align 4
  %6 = icmp eq i64 %len1, %maxLen2
  br i1 %6, label %if.then, label %if.else

if.then:                                          ; preds = %entry
  %factor = alloca i64, align 8
  %factor3 = getelementptr inbounds %string, %string* %2, i32 0, i32 1
  %7 = load i64, i64* %factor3, align 4
  store i64 %7, i64* %factor, align 4
  %factor4 = load i64, i64* %factor, align 4
  %8 = add i64 %maxLen2, %factor4
  call void @string_resize(%string* %2, i64 %8)
  br label %if.merge

if.else:                                          ; preds = %entry
  br label %if.merge

if.merge:                                         ; preds = %if.else, %if.then
  %buf = alloca i8*, align 8
  %buffer = getelementptr inbounds %string, %string* %2, i32 0, i32 0
  %9 = load i8*, i8** %buffer, align 8
  store i8* %9, i8** %buf, align 8
  %buf5 = load i8*, i8** %buf, align 8
  %10 = getelementptr i8*, i8** %buf, i64 %len1
  %11 = load i8*, i8** %10, align 8
  %12 = load i8, i8* %11, align 1
  store i8 %3, i8* %11, align 1
  ret void
}
