; ModuleID = 'main.ss'
source_filename = "main.ss"

%Person = type { i16, i32 }

define %Person* @test(%Person %p) {
entry:
  %0 = alloca %Person, align 8
  store %Person %p, %Person* %0, align 4
  %1 = load %Person, %Person* %0, align 4
  ret %Person* %0
}

define void @main() {
entry:
  %0 = alloca %Person, align 8
  %address = getelementptr inbounds %Person, %Person* %0, i32 0, i32 0
  store i16 5, i16* %address, align 2
  %age = getelementptr inbounds %Person, %Person* %0, i32 0, i32 1
  store i32 36, i32* %age, align 4
  %1 = load %Person, %Person* %0, align 4
  %p = alloca %Person*, align 8
  store %Person* %0, %Person** %p, align 8
  %p1 = load %Person*, %Person** %p, align 8
  %new = alloca %Person*, align 8
  %2 = call %Person* @test(%Person %1)
  store %Person* %2, %Person** %new, align 8
  %new2 = load %Person*, %Person** %new, align 8
  ret void
}
