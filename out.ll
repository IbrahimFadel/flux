; ModuleID = 'TheModule'
source_filename = "TheModule"

declare i32 @printf(i8*, ...)

define i32 @increaseNumber(i32 %num, i32 %increaseBy) {
entry:
  %increaseBy2 = alloca i32, align 4
  %num1 = alloca i32, align 4
  store i32 %num, i32* %num1, align 4
  %num1_loaded = load i32, i32* %num1, align 4
  store i32 %increaseBy, i32* %increaseBy2, align 4
  %increaseBy2_loaded = load i32, i32* %increaseBy2, align 4
  %num1_loaded3 = load i32, i32* %num1, align 4
  %increaseBy2_loaded4 = load i32, i32* %increaseBy2, align 4
  %addtmp = add i32 %num1_loaded3, %increaseBy2_loaded4
  ret i32 %addtmp
}

define i32 @main() {
entry:
  %formatString = alloca [32 x i8], align 1
  %0 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 0
  store i8 77, i8* %0, align 1
  %1 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 1
  store i8 121, i8* %1, align 1
  %2 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 2
  store i8 32, i8* %2, align 1
  %3 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 3
  store i8 97, i8* %3, align 1
  %4 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 4
  store i8 103, i8* %4, align 1
  %5 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 5
  store i8 101, i8* %5, align 1
  %6 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 6
  store i8 32, i8* %6, align 1
  %7 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 7
  store i8 105, i8* %7, align 1
  %8 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 8
  store i8 110, i8* %8, align 1
  %9 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 9
  store i8 32, i8* %9, align 1
  %10 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 10
  store i8 37, i8* %10, align 1
  %11 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 11
  store i8 100, i8* %11, align 1
  %12 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 12
  store i8 32, i8* %12, align 1
  %13 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 13
  store i8 121, i8* %13, align 1
  %14 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 14
  store i8 101, i8* %14, align 1
  %15 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 15
  store i8 97, i8* %15, align 1
  %16 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 16
  store i8 114, i8* %16, align 1
  %17 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 17
  store i8 115, i8* %17, align 1
  %18 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 18
  store i8 32, i8* %18, align 1
  %19 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 19
  store i8 119, i8* %19, align 1
  %20 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 20
  store i8 105, i8* %20, align 1
  %21 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 21
  store i8 108, i8* %21, align 1
  %22 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 22
  store i8 108, i8* %22, align 1
  %23 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 23
  store i8 32, i8* %23, align 1
  %24 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 24
  store i8 98, i8* %24, align 1
  %25 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 25
  store i8 101, i8* %25, align 1
  %26 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 26
  store i8 58, i8* %26, align 1
  %27 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 27
  store i8 32, i8* %27, align 1
  %28 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 28
  store i8 37, i8* %28, align 1
  %29 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 29
  store i8 100, i8* %29, align 1
  %30 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 30
  store i8 12, i8* %30, align 1
  %31 = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 31
  store i8 0, i8* %31, align 1
  %currentAge = alloca i32, align 4
  store i32 16, i32* %currentAge, align 4
  %yearsIntoTheFuture = alloca i32, align 4
  store i32 10, i32* %yearsIntoTheFuture, align 4
  %age = alloca i32, align 4
  %currentAge_loaded = load i32, i32* %currentAge, align 4
  %yearsIntoTheFuture_loaded = load i32, i32* %yearsIntoTheFuture, align 4
  %calltmp = call i32 @increaseNumber(i32 %currentAge_loaded, i32 %yearsIntoTheFuture_loaded)
  store i32 %calltmp, i32* %age, align 4
  %formatString_loaded = getelementptr inbounds [32 x i8], [32 x i8]* %formatString, i32 0, i32 0
  %yearsIntoTheFuture_loaded1 = load i32, i32* %yearsIntoTheFuture, align 4
  %age_loaded = load i32, i32* %age, align 4
  %calltmp2 = call i32 (i8*, ...) @printf(i8* %formatString_loaded, i32 %yearsIntoTheFuture_loaded1, i32 %age_loaded)
  ret i32 0
}
