define float @main() {
entry:
	%0 = alloca i32
	store i32 1, i32* %0
	%1 = load i32, i32* %0
	%2 = alloca i32
	store i32 2, i32* %2
	%3 = load i32, i32* %2
	%4 = alloca i32
	store i32 3, i32* %4
	%5 = load i32, i32* %4
	ret float 10.5
}
