; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @add(i32* dereferenceable(8) %num1, i32* dereferenceable(8) %num2) {
entry:
  %num22 = alloca i32*, align 8
  %num11 = alloca i32*, align 8
  store i32* %num1, i32** %num11, align 8
  %0 = load i32*, i32** %num11, align 8
  store i32* %num2, i32** %num22, align 8
  %1 = load i32*, i32** %num22, align 8
  store i32 23, i32* %0, align 4
  %2 = load i32*, i32** %num11, align 8
  %3 = load i32*, i32** %num22, align 8
  %4 = load i32, i32* %2, align 4
  %5 = load i32, i32* %3, align 4
  %addtmp = add i32 %4, %5
  ret i32 %addtmp
}

define i32 @main() {
entry:
  %x = alloca i32, align 4
  store i32 5, i32* %x, align 4
  %0 = load i32, i32* %x, align 4
  %z = alloca i32, align 4
  store i32 10, i32* %z, align 4
  %1 = load i32, i32* %z, align 4
  %sum = alloca i32, align 4
  %calltmp = call i32 @add(i32* dereferenceable(8) %x, i32* dereferenceable(8) %z)
  store i32 %calltmp, i32* %sum, align 4
  %2 = load i32, i32* %sum, align 4
  %3 = load i32, i32* %x, align 4
  ret i32 %3
}
