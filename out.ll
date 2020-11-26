; ModuleID = 'TheModule'
source_filename = "TheModule"

%Person = type { i16, i32 }

define %Person @get_person(i32 %x, i16 %y) {
entry:
  %new_person = alloca %Person, align 8
  %address_ptr = getelementptr inbounds %Person, %Person* %new_person, i64 0, i32 0
  store i16 %y, i16* %address_ptr, align 8
  %age_ptr = getelementptr inbounds %Person, %Person* %new_person, i64 0, i32 1
  store i32 %x, i32* %age_ptr, align 4
  %0 = load %Person, %Person* %new_person, align 8
  ret %Person %0
}

define %Person @main() {
entry:
  %calltmp = call %Person @get_person(i32 16, i16 123)
  ret %Person %calltmp
}
