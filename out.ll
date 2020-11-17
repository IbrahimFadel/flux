; ModuleID = 'TheModule'
source_filename = "TheModule"

%my_object = type { i16, i32 }

define %my_object @main() {
entry:
  %test = alloca %my_object, align 8
  %address_ptr = getelementptr inbounds %my_object, %my_object* %test, i32 0, i32 0
  store i16 5, i16* %address_ptr, align 2
  %fav_number_ptr = getelementptr inbounds %my_object, %my_object* %test, i32 0, i32 1
  store i32 10, i32* %fav_number_ptr, align 4
  %0 = load %my_object, %my_object* %test, align 4
  ret %my_object %0
}
