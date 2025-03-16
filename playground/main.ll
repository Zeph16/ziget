; ModuleID = 'ziget'
source_filename = "ziget"

@str = private unnamed_addr constant [11 x i8] c"Hello, %s\0A\00", align 1
@str.1 = private unnamed_addr constant [6 x i8] c"Ziget\00", align 1

declare i32 @printf(ptr, ...)

define void @greet_times(ptr %0, double %1) {
entry:
  %name = alloca ptr, align 8
  store ptr %0, ptr %name, align 8
  %times = alloca double, align 8
  store double %1, ptr %times, align 8
  %i = alloca double, align 8
  %times1 = load double, ptr %times, align 8
  store double %times1, ptr %i, align 8
  br label %loop

loop:                                             ; preds = %merge, %entry
  %i2 = load double, ptr %i, align 8
  %eqtmp = fcmp oeq double %i2, 0.000000e+00
  %cond = icmp eq i1 %eqtmp, true
  br i1 %cond, label %then, label %else

afterloop:                                        ; preds = %then
  ret void
  ret void

then:                                             ; preds = %loop
  br label %afterloop
  br label %merge

else:                                             ; preds = %loop
  br label %merge

merge:                                            ; preds = %else, %then
  %name3 = load ptr, ptr %name, align 8
  %printtmp = call i32 (ptr, ...) @printf(ptr @str, ptr %name3)
  %i4 = load double, ptr %i, align 8
  %subtmp = fsub double %i4, 1.000000e+00
  store double %subtmp, ptr %i, align 8
  br label %loop
}

define void @main() {
entry:
  call void @greet_times(ptr @str.1, double 3.000000e+00)
  ret void
}
