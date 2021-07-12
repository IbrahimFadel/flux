%Foo = type i32
%Animal = type { %Animal_VTable_Type* }
%Animal_VTable_Type = type {}
%Animal = type { %Animal_VTable_Type* }
%Dog = type { i32, i32*, i32*, i32*, i32, i32 }
%Dog = type { i32, i32*, i32*, i32*, i32, i32 }

@Animal_VTable_Data = global %Animal_VTable_Type {}

define i32 @Hello(%Dog* %dog) {
entry:
	ret i32 0
}

define void @Other(%Dog* %dog) {
entry:
	ret void
}

define i32 @main() {
entry:
	%0 = alloca %Animal
	%1 = load %Animal, %Animal* null
	store %Animal %1, %Animal* %0
	%2 = load %Animal, %Animal* %0
	%3 = alloca %Foo
	store %Foo 0, %Foo* %3
	%4 = load %Foo, %Foo* %3
	ret i32 0
}
