%Dog = type { i32, i32*, i32*, i32*, i32, i32 }

define %Dog @GetDog() {
entry:
	%0 = alloca %Dog
	%1 = load %Dog, %Dog* null
	store %Dog %1, %Dog* %0
	%2 = load %Dog, %Dog* %0
	ret %Dog %2
}

define i32 @main() {
entry:
	%0 = alloca %Dog
	%1 = load %Dog, %Dog* null
	store %Dog %1, %Dog* %0
	%2 = load %Dog, %Dog* %0
	ret i32 0
}
