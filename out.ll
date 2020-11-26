; ModuleID = 'TheModule'
source_filename = "TheModule"

define i32 @main() {
entry:
  %numbers = alloca [3 x i32], align 4
  %0 = getelementptr inbounds [3 x i32], [3 x i32]* %numbers, i32 0, i32 0
  store i32 1, i32* %0, align 4
  %1 = getelementptr inbounds [3 x i32], [3 x i32]* %numbers, i32 0, i32 1
  store i32 2, i32* %1, align 4
  %2 = getelementptr inbounds [3 x i32], [3 x i32]* %numbers, i32 0, i32 2
  store i32 3, i32* %2, align 4
  %doubles = alloca [3 x double], align 8
  %3 = getelementptr inbounds [3 x double], [3 x double]* %doubles, i32 0, i32 0
  store double 1.200000e+00, double* %3, align 8
  %4 = getelementptr inbounds [3 x double], [3 x double]* %doubles, i32 0, i32 1
  store double 3.900000e+00, double* %4, align 8
  %5 = getelementptr inbounds [3 x double], [3 x double]* %doubles, i32 0, i32 2
  store double 5.200000e+00, double* %5, align 8
  ret i32 0
}
