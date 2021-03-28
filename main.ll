; ModuleID = '/home/ibrahim/dev/sandscript/main.ss'
source_filename = "/home/ibrahim/dev/sandscript/main.ss"

%Person = type { i32, i16, i8, i64 }

define void @Person.constructor(%Person* %this, i32 %age, i16 %address, i8 %test, i64 %hi) {
entry:
  %this1 = alloca %Person*, align 8
  store %Person* %this, %Person** %this1, align 8
  %0 = load %Person*, %Person** %this1, align 8
  %age2 = alloca i32, align 4
  store i32 %age, i32* %age2, align 4
  %1 = load i32, i32* %age2, align 4
  %address3 = alloca i16, align 2
  store i16 %address, i16* %address3, align 2
  %2 = load i16, i16* %address3, align 2
  %test4 = alloca i8, align 1
  store i8 %test, i8* %test4, align 1
  %3 = load i8, i8* %test4, align 1
  %hi5 = alloca i64, align 8
  store i64 %hi, i64* %hi5, align 4
  %4 = load i64, i64* %hi5, align 4
  %5 = getelementptr inbounds %Person, %Person* %0, i32 0, i32 0
  %6 = load i32, i32* %5, align 4
  store i32 %1, i32* %5, align 4
  %7 = getelementptr inbounds %Person, %Person* %0, i32 0, i32 1
  %8 = load i16, i16* %7, align 2
  store i16 %2, i16* %7, align 2
  %9 = getelementptr inbounds %Person, %Person* %0, i32 0, i32 2
  %10 = load i8, i8* %9, align 1
  store i8 %3, i8* %9, align 1
  %11 = getelementptr inbounds %Person, %Person* %0, i32 0, i32 3
  %12 = load i64, i64* %11, align 4
  store i64 %4, i64* %11, align 4
  ret void
}

define void @Person.print(%Person* %this) {
entry:
  %this1 = alloca %Person*, align 8
  store %Person* %this, %Person** %this1, align 8
  %0 = load %Person*, %Person** %this1, align 8
  ret void
}

define i8 @Person.testFunction(%Person* %this, i8 %param) {
entry:
  %this1 = alloca %Person*, align 8
  store %Person* %this, %Person** %this1, align 8
  %0 = load %Person*, %Person** %this1, align 8
  %param2 = alloca i8, align 1
  store i8 %param, i8* %param2, align 1
  %1 = load i8, i8* %param2, align 1
  ret i8 0
}

define i32 @main() {
entry:
  %0 = alloca %Person*, align 8
  %1 = alloca %Person, align 8
  call void @Person.constructor(%Person* %1, i32 16, i16 5, i8 0, i64 0)
  %2 = load %Person, %Person* %1, align 4
  store %Person* %1, %Person** %0, align 8
  %person = load %Person*, %Person** %0, align 8
  ret i32 0
}
