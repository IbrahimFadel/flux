; ModuleID = '/home/ibrahim/dev/pi-lang/string.pi'
source_filename = "/home/ibrahim/dev/pi-lang/string.pi"

%string = type { i8*, i64, i64, i64 }

declare i8* @malloc(i32)

declare void @free(i8*)

define void @string.constructor(%string* %this) {
entry:
  %0 = getelementptr inbounds %string, %string* %this, i64 0, i32 0
  store i8* null, i8** %0, align 8
  %1 = getelementptr inbounds %string, %string* %this, i64 0, i32 1
  store i64 0, i64* %1, align 4
  %2 = getelementptr inbounds %string, %string* %this, i64 0, i32 2
  store i64 0, i64* %2, align 4
  %3 = getelementptr inbounds %string, %string* %this, i64 0, i32 3
  store i64 16, i64* %3, align 4
  ret void
}

define void @string.delete(%string* %this) {
entry:
  %0 = getelementptr inbounds %string, %string* %this, i64 0, i32 0
  %1 = load i8*, i8** %0, align 8
  %.not = icmp eq i8* %1, null
  br i1 %.not, label %if.merge, label %if.then

if.then:                                          ; preds = %entry
  call void @free(i8* %1)
  br label %if.merge

if.merge:                                         ; preds = %entry, %if.then
  ret void
}
