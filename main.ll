; ModuleID = '/home/ibrahim/dev/pi-lang/main.pi'
source_filename = "/home/ibrahim/dev/pi-lang/main.pi"

%Person = type { i32, i16, i8, i64 }

declare i8* @malloc(i32)

define void @Person.constructor(%Person* %this, i32 %age, i16 %address, i8 %test, i64 %hi) {
entry:
  %0 = getelementptr inbounds %Person, %Person* %this, i64 0, i32 0
  store i32 %age, i32* %0, align 4
  %1 = getelementptr inbounds %Person, %Person* %this, i64 0, i32 1
  store i16 %address, i16* %1, align 2
  %2 = getelementptr inbounds %Person, %Person* %this, i64 0, i32 2
  store i8 %test, i8* %2, align 1
  %3 = getelementptr inbounds %Person, %Person* %this, i64 0, i32 3
  store i64 %hi, i64* %3, align 4
  ret void
}

define void @Person.print(%Person* %this, i32 %x) {
entry:
  ret void
}

define i8 @Person.testFunction(%Person* %this, i8 %param) {
entry:
  ret i8 0
}

define i32 @main() {
entry:
  %0 = alloca %Person, align 8
  call void @Person.constructor(%Person* nonnull %0, i32 16, i16 5, i8 0, i64 0)
  call void @Person.print(%Person* nonnull %0, i32 1)
  ret i32 0
}
