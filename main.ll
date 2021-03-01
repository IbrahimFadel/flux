; ModuleID = '/home/ibrahim/dev/sandscript/main.ss'
source_filename = "/home/ibrahim/dev/sandscript/main.ss"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

define i32 @main() {
entry:
  %0 = alloca i32, align 4
  store i32 0, i32* %0, align 4
  %x = load i32, i32* %0, align 4
  %1 = icmp ult i32 %x, 10
  %2 = icmp eq i32 %x, 10
  %3 = or i1 %1, %2
  %4 = icmp eq i32 %x, 0
  %5 = and i1 %3, %4
  br i1 %5, label %if.then, label %if.merge

if.then:                                          ; preds = %entry
  store i32 20, i32* %0, align 4
  br label %if.merge

if.merge:                                         ; preds = %entry, %if.then
  store i32 50, i32* %0, align 4
  ret i32 %x
}
